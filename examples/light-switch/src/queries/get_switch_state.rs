use async_trait::async_trait;
use eventually::{aggregate, message, query};

use crate::application::LightSwitchService;
use crate::domain::{LightSwitch, LightSwitchId, LightSwitchRoot, LightSwitchState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSwitchState {
    pub id: LightSwitchId,
}

impl message::Message for GetSwitchState {
    fn name(&self) -> &'static str {
        "GetSwitch"
    }
}

#[async_trait]
impl<R> query::Handler<GetSwitchState> for LightSwitchService<R>
where
    R: aggregate::Repository<LightSwitch>,
{
    type Error = anyhow::Error;
    type Output = LightSwitchState;

    async fn handle(
        &self,
        query: query::Envelope<GetSwitchState>,
    ) -> Result<LightSwitchState, Self::Error> {
        let query = query.message;
        let root: LightSwitchRoot = self.light_switch_repository.get(&query.id).await?.into();
        let s = root.get_switch_state()?;
        Ok(s)
    }
}
