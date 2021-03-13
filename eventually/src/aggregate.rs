use std::convert::{TryFrom, TryInto};
use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::ops::DerefMut;

use futures::stream::TryStreamExt;

use crate::eventstore::{
    ConflictError, EventStore, IntoConflictError, Select, Stream, VersionCheck,
};
use crate::{Event, Events};

pub trait Aggregate: Sized {
    type Id: TryFrom<String> + Display;
    type Event;
    type Error;

    fn id(&self) -> &Self::Id;
    fn apply_first(event: Self::Event) -> Result<Self, Self::Error>;
    fn apply(&mut self, event: Self::Event) -> Result<(), Self::Error>;
}

pub trait AggregateRoot<A>: From<Context<A>> + DerefMut<Target = Context<A>>
where
    A: Aggregate,
    A::Event: Clone,
{
    fn new(event: Event<A::Event>) -> Result<Self, A::Error> {
        Ok(Self::from(Context::<A>::new(event, true)?))
    }
}

impl<T, A> AggregateRoot<A> for T
where
    A: Aggregate,
    A::Event: Clone,
    T: From<Context<A>> + DerefMut<Target = Context<A>>,
{
}

#[derive(Debug, Clone)]
pub struct Context<A>
where
    A: Aggregate,
{
    aggregate: A,
    recorded_events: Events<A::Event>,
    version: i64,
}

impl<A> Context<A>
where
    A: Aggregate,
    A::Event: Clone,
{
    #[inline]
    pub fn version(&self) -> i64 {
        self.version
    }

    #[inline]
    pub fn record_that(&mut self, event: Event<A::Event>) -> Result<(), A::Error> {
        self.record_event(event.payload.clone())?;
        self.recorded_events.push(event);
        Ok(())
    }

    #[inline]
    pub fn aggregate(&self) -> &A {
        &self.aggregate
    }

    fn new(event: Event<A::Event>, record_event: bool) -> Result<Self, A::Error> {
        let aggregate = A::apply_first(event.payload.clone())?;
        let mut recorded_events = Vec::new();

        if record_event {
            recorded_events.push(event);
        }

        Ok(Self {
            version: 1,
            aggregate,
            recorded_events,
        })
    }

    #[inline]
    fn record_event(&mut self, event: A::Event) -> Result<(), A::Error> {
        self.aggregate.apply(event)?;
        self.version += 1;
        Ok(())
    }

    #[inline]
    fn flush_recorded_events(&mut self) -> Events<A::Event> {
        std::mem::replace(&mut self.recorded_events, Vec::new())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError<A>
where
    A: StdError + 'static,
{
    #[error("failed to rehydrate aggregate: {0}")]
    Rehydrate(#[source] A),

    #[error("failed to append to event store due to conflict: {0}")]
    Conflict(#[from] ConflictError),

    #[error("failed to convert from event store to aggregate event: {0}")]
    AggregateEventConversion(#[source] anyhow::Error),

    #[error("error while streaming from event store: {0}")]
    StreamError(#[source] anyhow::Error),

    #[error("failed to append to event store: {0}")]
    AppendToEventStore(#[source] anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct Repository<A, R, ES>
where
    A: Aggregate,
    A::Event: Clone,
    R: AggregateRoot<A>,
    ES: EventStore,
    ES::Event: TryInto<A::Event> + From<A::Event>,
{
    event_store: ES,
    aggregate_type_name: String,
    aggregate: std::marker::PhantomData<A>,
    aggregate_root: std::marker::PhantomData<R>,
}

impl<A, R, ES> Repository<A, R, ES>
where
    A: Aggregate,
    A::Event: Clone,
    A::Error: StdError + 'static,
    R: AggregateRoot<A>,
    ES: EventStore,
    ES::Event: TryInto<A::Event> + From<A::Event>,
    <ES::Event as TryInto<A::Event>>::Error: StdError + Send + Sync + 'static,
{
    pub fn new(aggregate_type_name: String, event_store: ES) -> Self {
        Self {
            event_store,
            aggregate_type_name,
            aggregate: std::marker::PhantomData,
            aggregate_root: std::marker::PhantomData,
        }
    }

    pub async fn get(&self, id: &A::Id) -> Result<Option<R>, RepositoryError<A::Error>> {
        let id = id.to_string();

        let events = self.event_store.stream(
            Stream::Id {
                id: &id,
                category: &self.aggregate_type_name,
            },
            Select::All,
        );

        events
            .map_err(anyhow::Error::from)
            .map_err(RepositoryError::StreamError)
            // Cast from Event Store event into Aggregate event.
            //
            // NOTE: this is necessary as the Event Store may use
            // a "bigger" number of events than the one necessary
            // for the current aggregate.
            .try_filter_map(|evt| async move {
                let casted_event: A::Event = evt
                    .event
                    .payload
                    .try_into()
                    .map_err(anyhow::Error::from)
                    .map_err(RepositoryError::AggregateEventConversion)?;

                Ok(Some(casted_event))
            })
            // Rehydrate the Aggregate state from the incoming Event Stream.
            .try_fold(None, |ctx: Option<Context<A>>, event| async {
                ctx.map(|mut ctx| ctx.record_event(event.clone()).map(|_| ctx))
                    .or_else(|| Some(Context::new(Event::from(event), false)))
                    .transpose()
                    .map_err(RepositoryError::Rehydrate)
            })
            .await
            // Create an Aggregate Root instance from the rehydrated context.
            .map(|ctx| ctx.map(R::from))
    }

    pub async fn add(&mut self, root: &mut R) -> Result<(), RepositoryError<A::Error>> {
        let new_events = root.flush_recorded_events();

        if new_events.is_empty() {
            return Ok(());
        }

        let id = root.aggregate().id().to_string();

        let mapped_new_events: Events<ES::Event> = new_events
            .into_iter()
            .map(|event| Event {
                payload: ES::Event::from(event.payload),
                metadata: event.metadata,
            })
            .collect();

        let expected_version =
            VersionCheck::Exact(root.version() - (mapped_new_events.len() as i64));

        self.event_store
            .append(
                &self.aggregate_type_name,
                &id,
                expected_version,
                mapped_new_events,
            )
            .await
            .map_err(|e| {
                e.into_conflict_error()
                    .map(RepositoryError::Conflict)
                    .unwrap_or_else(|| RepositoryError::AppendToEventStore(e.into()))
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    use crate::inmemory::InMemoryEventStore;

    #[derive(Debug, Clone)]
    enum OrderEvent {
        WasCreated { order_id: String },
        ItemWasAdded { item_sku: String },
        WasCompleted,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    enum OrderError {
        #[error("order is already created")]
        AlreadyCreated,

        #[error("order is not created yet")]
        NotYetCreated,

        #[error("item already added to the order")]
        ItemAlreadyAdded,

        #[error("order is already marked as complete")]
        MarkedAsComplete,
    }

    #[derive(Debug)]
    struct Order {
        order_id: String,
        items_sku: Vec<String>,
        is_completed: bool,
    }

    impl Order {
        fn new(order_id: String) -> Self {
            Self {
                order_id,
                items_sku: Vec::new(),
                is_completed: false,
            }
        }
    }

    impl Aggregate for Order {
        type Id = String;
        type Event = OrderEvent;
        type Error = OrderError;

        fn id(&self) -> &Self::Id {
            &self.order_id
        }

        fn apply_first(event: Self::Event) -> Result<Self, Self::Error> {
            match event {
                OrderEvent::WasCreated { order_id } => Ok(Self::new(order_id)),
                _ => Err(OrderError::NotYetCreated),
            }
        }

        fn apply(&mut self, event: Self::Event) -> Result<(), Self::Error> {
            match event {
                OrderEvent::WasCreated { .. } => return Err(OrderError::AlreadyCreated),
                OrderEvent::ItemWasAdded { item_sku } => self.items_sku.push(item_sku),
                OrderEvent::WasCompleted => self.is_completed = true,
            };

            Ok(())
        }
    }

    #[derive(Debug)]
    struct OrderRoot(Context<Order>);

    impl From<Context<Order>> for OrderRoot {
        #[inline]
        fn from(ctx: Context<Order>) -> Self {
            Self(ctx)
        }
    }

    impl Deref for OrderRoot {
        type Target = Context<Order>;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for OrderRoot {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl OrderRoot {
        pub fn create_new_order(order_id: String) -> Result<Self, OrderError> {
            AggregateRoot::<Order>::new(OrderEvent::WasCreated { order_id }.into())
        }

        pub fn add_new_item(mut self, item_sku: String) -> Result<Self, OrderError> {
            if self.aggregate().is_completed {
                return Err(OrderError::MarkedAsComplete);
            }

            let item_is_already_added = self.aggregate().items_sku.iter().any(|it| it == &item_sku);

            if item_is_already_added {
                return Err(OrderError::ItemAlreadyAdded);
            }

            self.record_that(OrderEvent::ItemWasAdded { item_sku }.into())?;

            Ok(self)
        }

        pub fn mark_as_complete(mut self) -> Result<Self, OrderError> {
            if self.aggregate().is_completed {
                return Err(OrderError::MarkedAsComplete);
            }

            self.record_that(OrderEvent::WasCompleted.into())?;

            Ok(self)
        }
    }

    #[tokio::test]
    async fn it_works() {
        let mut order = OrderRoot::create_new_order("my-order".to_owned())
            .unwrap()
            .add_new_item("DE-CB-3-2-0".to_owned())
            .unwrap()
            .mark_as_complete()
            .unwrap();

        println!("Order: {:?}", order);

        let event_store = InMemoryEventStore::<OrderEvent>::default();
        let mut repository = Repository::new("order".to_owned(), event_store.clone());
        repository.add(&mut order).await.unwrap();

        let persisted_order = repository.get(order.aggregate().id()).await.unwrap();

        println!("Order: {:?}", order);
        println!("[Recovered] Order: {:?}", persisted_order);
        println!("Event store: {:?}", event_store);

        let result = persisted_order.unwrap().mark_as_complete();
        println!("Marked as complete twice: \"{}\"", result.unwrap_err());
    }
}
