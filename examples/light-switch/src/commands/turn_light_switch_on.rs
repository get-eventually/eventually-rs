use async_trait::async_trait;
use eventually::{aggregate, command, message};

use crate::application::LightSwitchService;
use crate::domain::{LightSwitch, LightSwitchId, LightSwitchRoot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnLightSwitchOn {
    pub id: LightSwitchId,
}

impl message::Message for TurnLightSwitchOn {
    fn name(&self) -> &'static str {
        "TurnLightSwitchOn"
    }
}

#[async_trait]
impl<R> command::Handler<TurnLightSwitchOn> for LightSwitchService<R>
where
    R: aggregate::Repository<LightSwitch>,
{
    type Error = anyhow::Error;
    async fn handle(
        &self,
        command: command::Envelope<TurnLightSwitchOn>,
    ) -> Result<(), Self::Error> {
        let command = command.message;
        let mut root: LightSwitchRoot = self.light_switch_repository.get(&command.id).await?.into();
        let _ = root.turn_on(command.id)?;
        self.light_switch_repository.save(&mut root).await?;
        Ok(())
    }
}
