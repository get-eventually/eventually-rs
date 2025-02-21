use async_trait::async_trait;
use eventually::{aggregate, message, query};

use crate::application::LightswitchService;
use crate::domain::{Lightswitch, LightswitchId, LightswitchRoot, LightswitchState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSwitchState {
    pub id: LightswitchId,
}

impl message::Message for GetSwitchState {
    fn name(&self) -> &'static str {
        "GetSwitch"
    }
}

#[async_trait]
impl<R> query::Handler<GetSwitchState> for LightswitchService<R>
where
    R: aggregate::Repository<Lightswitch>,
{
    type Error = anyhow::Error;
    type Output = LightswitchState;

    async fn handle(
        &self,
        query: query::Envelope<GetSwitchState>,
    ) -> Result<LightswitchState, Self::Error> {
        let query = query.message;
        let root: LightswitchRoot = self.light_switch_repository.get(&query.id).await?.into();
        let s = root.get_switch_state()?;
        Ok(s)
    }
}
