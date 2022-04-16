use std::{
    borrow::{Borrow, BorrowMut},
    fmt::Debug,
};

#[allow(unused_imports)] // The `aggregate` package is used by the documentation.
use crate::{aggregate, aggregate::Aggregate, event, version::Version};

/// A context object that should be used by the Aggregate [Root] methods to
/// access the [Aggregate] state and to record new Domain Events.
#[derive(Debug, Clone)]
#[must_use]
pub struct Context<T>
where
    T: Aggregate,
{
    aggregate: T,
    version: Version,
    recorded_events: Vec<event::Envelope<T::Event>>,
}

impl<T> Context<T>
where
    T: Aggregate,
{
    /// Returns the current version for the [Aggregate].
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the list of uncommitted, recorded Domain [Event]s from the [Context]
    /// and resets the internal list to its default value.
    #[doc(hidden)]
    pub fn take_uncommitted_events(&mut self) -> Vec<event::Envelope<T::Event>> {
        std::mem::take(&mut self.recorded_events)
    }

    /// Creates a new [Context] instance from a Domain [Event]
    /// while rehydrating an [Aggregate].
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    pub(crate) fn rehydrate_from(event: event::Envelope<T::Event>) -> Result<Context<T>, T::Error> {
        Ok(Context {
            version: 1,
            aggregate: T::apply(None, event.message)?,
            recorded_events: Vec::default(),
        })
    }

    /// Applies a new Domain [Event] to the [Context] while rehydrating
    /// an [Aggregate].
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    pub(crate) fn apply_rehydrated_event(
        mut self,
        event: event::Envelope<T::Event>,
    ) -> Result<Context<T>, T::Error> {
        self.aggregate = T::apply(Some(self.aggregate), event.message)?;
        self.version += 1;

        Ok(self)
    }

    /// Returns read access to the [Aggregate] state.
    fn state(&self) -> &T {
        &self.aggregate
    }

    fn record_new(event: event::Envelope<T::Event>) -> Result<Context<T>, T::Error> {
        Ok(Context {
            version: 1,
            aggregate: T::apply(None, event.message.clone())?,
            recorded_events: vec![event],
        })
    }

    fn record_that(&mut self, event: event::Envelope<T::Event>) -> Result<(), T::Error> {
        self.aggregate = T::apply(Some(self.aggregate.clone()), event.message.clone())?;
        self.recorded_events.push(event);
        self.version += 1;

        Ok(())
    }
}

/// An Aggregate Root represents the Domain Entity object used to
/// load and save an [Aggregate] from and to an [`aggregate::Repository`], and
/// to perform actions that may result in new Domain Events
/// to change the state of the Aggregate.
///
/// An Aggregate Root implementation should only depend on [Context],
/// and implement the `From<Context<AggregateType>>` trait. The Aggregate state
/// and list of Domain Events recorded are handled by the Context object itself.
///
/// ```text
/// #[derive(Debug, Clone)]
/// struct MyAggregateRoot(Context<MyAggregate>);
///
/// impl From<Context<MyAggregate>> for MyAggregateRoot {
///     fn from(ctx: Context<MyAggregate>) -> Self {
///         Self(ctx)
///     }
/// }
///
/// // Implement the Aggregate Root interface by providing
/// // read/write access to the Context object.
/// impl aggregate::Root<MyAggregate> for MyAggregateRoot {
///     fn ctx(&self) -> &Context<MyAggregate> {
///         &self.0
///     }
///
///     fn ctx_mut(&mut self) -> &mut Context<MyAggregate> {
///         &mut self.0
///     }
/// }
/// ```
///
/// For more information on how to record Domain Events using an Aggregate Root,
/// please check [`Context::record_that`] method documentation.
pub trait Root<T>:
    From<Context<T>> + Borrow<Context<T>> + BorrowMut<Context<T>> + Send + Sync
where
    T: Aggregate,
{
    /// Provides read access to an [Aggregate] [Root] [Context].
    #[doc(hidden)]
    fn ctx(&self) -> &Context<T> {
        self.borrow()
    }

    /// Provides write access to an [Aggregate] [Root] [Context].
    #[doc(hidden)]
    fn ctx_mut(&mut self) -> &mut Context<T> {
        self.borrow_mut()
    }

    /// Provides convenient access to the [Aggregate] Root state.
    fn state(&self) -> &T {
        self.ctx().state()
    }

    /// Returns the unique identifier for the Aggregate instance.
    fn aggregate_id<'a>(&'a self) -> &'a T::Id
    where
        T: 'a,
    {
        self.state().aggregate_id()
    }

    /// Creates a new [Aggregate] [Root] instance by applying the specified
    /// Domain Event.
    ///
    /// Example of usage:
    /// ```text
    /// use eventually::{
    ///     event,
    ///     aggregate::Root,
    ///     aggregate,
    /// };
    ///
    /// let my_aggregate_root = MyAggregateRoot::record_new(
    ///     event::Envelope::from(MyDomainEvent { /* something */ })
    ///  )?;
    /// ```
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    fn record_new(event: event::Envelope<T::Event>) -> Result<Self, T::Error> {
        Context::record_new(event).map(Self::from)
    }

    /// Records a change to the [Aggregate] [Root], expressed by the specified
    /// Domain Event.
    ///
    /// Example of usage:
    /// ```text
    /// use eventually::{
    ///     event,
    ///     aggregate::Root,
    /// };
    ///
    /// impl MyAggregateRoot {
    ///     pub fn update_name(&mut self, name: String) -> Result<(), MyAggregateError> {
    ///         if name.is_empty() {
    ///             return Err(MyAggregateError::NameIsEmpty);
    ///         }
    ///
    ///         self.record_that(
    ///             event::Envelope::from(MyAggergateEvent::NameWasChanged { name })
    ///         )
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    fn record_that(&mut self, event: event::Envelope<T::Event>) -> Result<(), T::Error> {
        self.ctx_mut().record_that(event)
    }
}
