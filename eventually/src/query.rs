//! Module `query` contains types and helpful abstractions to model Domain Queries
//! and implement Domain Query Handlers.

use async_trait::async_trait;
use futures::Future;

use crate::message;

/// A [Message][message::Message] carrying the Domain Query itself as payload
/// and other relevant information as metadata.
pub type Envelope<T> = message::Envelope<T>;

/// An Handler describes an implementation that is able to handle specific [Queries][Envelope].
///
/// The Handler evaluates the Domain Query and produces a **result**, here described
/// through the [Output][Handler::Output] associated type.
#[async_trait]
pub trait Handler<T>: Send + Sync
where
    T: message::Message + Send + Sync,
{
    /// The result type the Handler produces when evaluating a Query.
    type Output: Send + Sync;
    /// The error type returned by the Handler when Query evaluation fails.
    type Error: Send + Sync;

    /// Evaluates the [Query][Envelope] provided and returns a result type,
    /// described by the [Output][Handler::Output] parameter.
    ///
    /// # Errors
    ///
    /// As the Handler can fail to evaluate the Query, an [Error][Handler::Error]
    /// can be returned instead.
    async fn handle(&self, query: Envelope<T>) -> Result<Self::Output, Self::Error>;
}

#[async_trait]
impl<T, R, Err, F, Fut> Handler<T> for F
where
    T: message::Message + Send + Sync + 'static,
    R: Send + Sync,
    Err: Send + Sync,
    F: Send + Sync + Fn(Envelope<T>) -> Fut,
    Fut: Send + Sync + Future<Output = Result<R, Err>>,
{
    type Output = R;
    type Error = Err;

    async fn handle(&self, command: Envelope<T>) -> Result<R, Self::Error> {
        self(command).await
    }
}
