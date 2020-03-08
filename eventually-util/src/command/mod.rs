pub mod dispatcher;
pub mod r#static;

use async_trait::async_trait;

use eventually_core::aggregate::StateOf;
use eventually_core::command::{AggregateOf, CommandOf, Handler};

#[async_trait]
pub trait Dispatcher {
    type CommandHandler: Handler;
    type Error;

    async fn dispatch(
        &mut self,
        command: CommandOf<Self::CommandHandler>,
    ) -> Result<StateOf<AggregateOf<Self::CommandHandler>>, Self::Error>;
}
