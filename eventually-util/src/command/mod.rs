pub mod dispatcher;
pub mod r#static;

use futures::future::BoxFuture;

use eventually_core::aggregate::StateOf;
use eventually_core::command::{AggregateOf, CommandOf, Handler};

pub trait Dispatcher {
    type CommandHandler: Handler;
    type Error;

    fn dispatch(
        &mut self,
        command: CommandOf<Self::CommandHandler>,
    ) -> BoxFuture<Result<StateOf<AggregateOf<Self::CommandHandler>>, Self::Error>>;
}
