//! Contains an super-type that decorates a [Repository] implementation
//! to return opaque errors using [anyhow::Error].
//!
//! Check out [AnyRepository] for more information.

use std::marker::PhantomData;

use async_trait::async_trait;

use crate::aggregate::repository::{self, Repository};
use crate::aggregate::{self, Aggregate};

/// Represents a generic, opaque kind of error. Powered by [anyhow::Error],
/// but implemented in its own type to satisfy [std::error::Error] trait.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct AnyError(#[from] anyhow::Error);

/// A [Repository] trait that uses opaque errors through [AnyError],
/// rather than requiring concrete error types for [Repository::get] and [Repository::save].
///
/// This trait makes it easier to be used with trait objects, like `Arc<dyn AnyRepository<T>>`.
///
/// Use [AnyRepositoryExt::with_any_errors] to make use of this trait.
pub trait AnyRepository<T>: Repository<T, GetError = AnyError, SaveError = AnyError>
where
    T: Aggregate,
{
}

/// Extension trait for [Repository] instances which errors implement [std::error::Error],
/// that can be used to convert to an [AnyRepository] instance instead.
pub trait AnyRepositoryExt<T>: Sized + Repository<T>
where
    T: Aggregate,
    <Self as Repository<T>>::GetError: std::error::Error + Send + Sync + 'static,
    <Self as Repository<T>>::SaveError: std::error::Error + Send + Sync + 'static,
{
    /// Converts the current [Repository] instance into an [AnyRepository] implementation.
    #[must_use]
    fn with_any_errors(self) -> AnyRepositoryImpl<T, Self> {
        AnyRepositoryImpl {
            inner: self,
            aggregate_type: PhantomData,
        }
    }
}

impl<T, R> AnyRepositoryExt<T> for R
where
    T: Aggregate,
    R: Repository<T>,
    <R as Repository<T>>::GetError: std::error::Error + Send + Sync + 'static,
    <R as Repository<T>>::SaveError: std::error::Error + Send + Sync + 'static,
{
}

/// The concrete super-type implementation that decorates a [Repository]
/// to return opaque errors using [AnyError].
///
/// Must use the [AnyRepositoryExt] super-trait extension to obtain an instance
/// of this type.
pub struct AnyRepositoryImpl<T, R>
where
    T: Aggregate,
    R: Repository<T>,
    <R as Repository<T>>::GetError: std::error::Error + Send + Sync + 'static,
    <R as Repository<T>>::SaveError: std::error::Error + Send + Sync + 'static,
{
    inner: R,
    aggregate_type: PhantomData<T>,
}

#[async_trait]
impl<T, R> Repository<T> for AnyRepositoryImpl<T, R>
where
    T: Aggregate,
    R: Repository<T>,
    <R as Repository<T>>::GetError: std::error::Error + Send + Sync + 'static,
    <R as Repository<T>>::SaveError: std::error::Error + Send + Sync + 'static,
{
    type GetError = AnyError;
    type SaveError = AnyError;

    async fn get(
        &self,
        id: &T::Id,
    ) -> Result<aggregate::Root<T>, repository::GetError<Self::GetError>> {
        self.inner.get(id).await.map_err(|err| match err {
            repository::GetError::NotFound => repository::GetError::NotFound,
            repository::GetError::Inner(v) => {
                repository::GetError::Inner(anyhow::Error::from(v).into())
            },
        })
    }

    async fn save(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::SaveError> {
        Ok(self.inner.save(root).await.map_err(anyhow::Error::from)?)
    }
}

impl<T, R> AnyRepository<T> for AnyRepositoryImpl<T, R>
where
    T: Aggregate,
    R: Repository<T>,
    <R as Repository<T>>::GetError: std::error::Error + Send + Sync + 'static,
    <R as Repository<T>>::SaveError: std::error::Error + Send + Sync + 'static,
{
}
