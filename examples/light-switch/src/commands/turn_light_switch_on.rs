use async_trait::async_trait;
use eventually::{aggregate, command, message};

use crate::application::LightswitchService;
use crate::domain::{Lightswitch, LightswitchId, LightswitchRoot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnLightSwitchOn {
    pub id: LightswitchId,
}

impl message::Message for TurnLightSwitchOn {
    fn name(&self) -> &'static str {
        "TurnLightSwitchOn"
    }
}

#[async_trait]
impl<R> command::Handler<TurnLightSwitchOn> for LightswitchService<R>
where
    R: aggregate::Repository<Lightswitch>,
{
    type Error = anyhow::Error;
    async fn handle(
        &self,
        command: command::Envelope<TurnLightSwitchOn>,
    ) -> Result<(), Self::Error> {
        let command = command.message;
        let mut root: LightswitchRoot = self.light_switch_repository.get(&command.id).await?.into();
        let _ = root.turn_on(command.id)?;
        self.light_switch_repository.save(&mut root).await?;
        Ok(())
    }
}
