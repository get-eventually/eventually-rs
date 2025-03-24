use async_trait::async_trait;
use eventually::{aggregate, command, message};

use crate::application::LightswitchService;
use crate::domain::{Lightswitch, LightswitchId, LightswitchRoot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallLightSwitch {
    pub id: LightswitchId,
}

impl message::Message for InstallLightSwitch {
    fn name(&self) -> &'static str {
        "InstallLightSwitch"
    }
}

#[async_trait]
impl<R> command::Handler<InstallLightSwitch> for LightswitchService<R>
where
    R: aggregate::Repository<Lightswitch>,
{
    type Error = anyhow::Error;
    async fn handle(
        &self,
        command: command::Envelope<InstallLightSwitch>,
    ) -> Result<(), Self::Error> {
        let command = command.message;
        let mut light_switch = LightswitchRoot::install(command.id)?;
        self.light_switch_repository.save(&mut light_switch).await?;
        Ok(())
    }
}
