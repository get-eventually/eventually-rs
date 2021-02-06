use std::hash::Hash;
use std::ops::DerefMut;

pub trait Aggregate: Sized {
    type Id;
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
    fn new(event: A::Event) -> Result<Self, A::Error> {
        Ok(Self::from(Context::<A>::new(event)?))
    }
}

impl<T, A> AggregateRoot<A> for T
where
    A: Aggregate,
    A::Event: Clone,
    T: From<Context<A>> + DerefMut<Target = Context<A>>,
{
}

#[derive(Debug)]
pub struct Context<A>
where
    A: Aggregate,
{
    aggregate: A,
    recorded_events: Vec<A::Event>,
    version: u64,
}

impl<A> Context<A>
where
    A: Aggregate,
    A::Event: Clone,
{
    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn record_that(&mut self, event: A::Event) -> Result<(), A::Error> {
        self.record_event(event.clone())?;
        self.recorded_events.push(event);
        Ok(())
    }

    #[inline]
    pub fn aggregate(&self) -> &A {
        &self.aggregate
    }

    #[inline]
    fn new(event: A::Event) -> Result<Self, A::Error> {
        let aggregate = A::apply_first(event.clone())?;
        let mut recorded_events = Vec::new();
        recorded_events.push(event);

        Ok(Self {
            version: 1,
            aggregate,
            recorded_events,
        })
    }

    #[inline]
    fn rehydrate(mut events: impl Iterator<Item = A::Event>) -> Result<Option<Self>, A::Error> {
        events.try_fold(None, |ctx, event| {
            if ctx.is_none() {
                return Context::new(event).map(Some);
            }

            let mut ctx = ctx.unwrap();
            ctx.record_event(event)?;
            Ok(Some(ctx))
        })
    }

    #[inline]
    fn record_event(&mut self, event: A::Event) -> Result<(), A::Error> {
        self.aggregate.apply(event)?;
        self.version += 1;
        Ok(())
    }

    #[inline]
    fn flush_recorded_events(&mut self) -> Vec<A::Event> {
        std::mem::replace(&mut self.recorded_events, Vec::new())
    }
}

#[derive(Debug, Default)]
pub struct Repository<A, R>
where
    A: Aggregate,
    A::Event: Clone,
    R: AggregateRoot<A>,
{
    events: std::collections::HashMap<A::Id, Vec<A::Event>>,
    aggregate: std::marker::PhantomData<A>,
    aggregate_root: std::marker::PhantomData<R>,
}

impl<A, R> Repository<A, R>
where
    A: Aggregate,
    A::Id: Eq + Hash + Clone,
    A::Event: Clone,
    R: AggregateRoot<A>,
{
    pub fn new() -> Self {
        Self {
            events: std::collections::HashMap::new(),
            aggregate: std::marker::PhantomData,
            aggregate_root: std::marker::PhantomData,
        }
    }

    pub fn get(&self, id: &A::Id) -> Result<Option<R>, A::Error> {
        let events = match self.events.get(id) {
            None => return Ok(None),
            Some(events) => events,
        };

        Ok(Context::<A>::rehydrate(events.iter().cloned())?.map(R::from))
    }

    pub fn add(&mut self, root: &mut R) -> Result<(), A::Error> {
        let mut new_events = root.flush_recorded_events();

        if new_events.is_empty() {
            return Ok(());
        }

        self.events
            .entry(root.aggregate().id().clone())
            .and_modify(|events| events.append(&mut new_events))
            .or_insert(new_events);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    #[derive(Debug, Clone)]
    enum OrderEvent {
        WasCreated { order_id: String },
        ItemWasAdded { item_sku: String },
        WasCompleted,
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
        type Error = ();

        fn id(&self) -> &Self::Id {
            &self.order_id
        }

        fn apply_first(event: Self::Event) -> Result<Self, Self::Error> {
            match event {
                OrderEvent::WasCreated { order_id } => Ok(Self::new(order_id)),
                _ => Err(()),
            }
        }

        fn apply(&mut self, event: Self::Event) -> Result<(), Self::Error> {
            match event {
                OrderEvent::WasCreated { .. } => return Err(()),
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
        pub fn create_new_order(order_id: String) -> Result<Self, ()> {
            AggregateRoot::<Order>::new(OrderEvent::WasCreated { order_id })
        }

        pub fn add_new_item(&mut self, item_sku: String) -> Result<(), ()> {
            let item_is_already_added = self.aggregate().items_sku.iter().any(|it| it == &item_sku);
            if item_is_already_added {
                return Err(());
            }

            self.record_that(OrderEvent::ItemWasAdded { item_sku })
        }

        pub fn mark_as_complete(&mut self) -> Result<(), ()> {
            if self.aggregate().is_completed {
                return Err(());
            }

            self.record_that(OrderEvent::WasCompleted)
        }
    }

    #[test]
    fn it_works() {
        let mut order = OrderRoot::create_new_order("my-order".to_owned()).unwrap();
        order.add_new_item("DE-CB-3-2-0".to_owned()).unwrap();
        order.mark_as_complete().unwrap();

        println!("Order: {:?}", order);

        let mut repository = Repository::new();
        repository.add(&mut order).unwrap();

        println!("Order: {:?}", order);
        println!("Repository: {:?}", repository);
    }
}
