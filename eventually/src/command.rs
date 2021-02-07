use futures::future::BoxFuture;

pub trait CommandHandler<T> {
    type Error;

    fn handle(&mut self, command: T) -> BoxFuture<Result<(), Self::Error>>;
}
