pub mod repository;

use async_trait::async_trait;

use crate::version;

pub trait Named: Send + Sync {
    /// A unique name identifier for an Entity type.
    fn type_name() -> &'static str;
}

pub trait Identifiable: Send + Sync {
    /// The type used to uniquely identify the Entity.
    type Id: Eq + Send + Sync;

    /// Returns the unique identifier for the [Entity] instance.
    fn id(&self) -> &Self::Id;
}

pub trait Entity: Named + Identifiable + Sized {
    fn version(&self) -> version::Version;
}

/// Error returned by a call to [`Repository::get`].
/// This type is used to check whether an Aggregate Root has been found or not.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GetError<I> {
    /// This error is retured by [`Getter::get`] when the
    /// desired [`Entity`] could not be found in the data store.
    #[error("entity was not found")]
    EntityNotFound,

    /// An internal error that occurred when performing [`Getter::get`].
    #[error(transparent)]
    Other(#[from] I),
}

#[async_trait]
pub trait Getter<T>: Send + Sync
where
    T: Entity,
{
    /// The error type returned when the [`Getter::get`] method fails.
    type Error: Send + Sync;

    /// Loads an Entity instance from the data store, referenced by its unique identifier.
    async fn get(&self, id: &T::Id) -> Result<T, GetError<Self::Error>>;
}

#[async_trait]
pub trait Saver<T>: Send + Sync
where
    T: Entity,
{
    /// The error type returned when the [`Saver::save`] method fails.
    type Error: Send + Sync;

    /// Saves a new version of an [`Entity`] instance to the data store.
    async fn save(&self, entity: &mut T) -> Result<(), Self::Error>;
}

/// A Repository is an object that allows to load and save
/// an [Entity] from and to a persistent data store.
pub trait Repository<T>: Getter<T> + Saver<T> + Send + Sync
where
    T: Entity,
{
}

impl<T, R> Repository<T> for R
where
    T: Entity,
    R: Getter<T> + Saver<T> + Send + Sync,
{
}
