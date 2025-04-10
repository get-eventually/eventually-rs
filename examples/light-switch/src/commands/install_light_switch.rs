use async_trait::async_trait;
use eventually::{aggregate, command, message};

use crate::application::LightSwitchService;
use crate::domain::{LightSwitch, LightSwitchId, LightSwitchRoot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallLightSwitch {
    pub id: LightSwitchId,
}

impl message::Message for InstallLightSwitch {
    fn name(&self) -> &'static str {
        "InstallLightSwitch"
    }
}

#[async_trait]
impl<R> command::Handler<InstallLightSwitch> for LightSwitchService<R>
where
    R: aggregate::Repository<LightSwitch>,
{
    type Error = anyhow::Error;
    async fn handle(
        &self,
        command: command::Envelope<InstallLightSwitch>,
    ) -> Result<(), Self::Error> {
        let command = command.message;
        let mut light_switch = LightSwitchRoot::install(command.id)?;
        self.light_switch_repository.save(&mut light_switch).await?;
        Ok(())
    }
}
