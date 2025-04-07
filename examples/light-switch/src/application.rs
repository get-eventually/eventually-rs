use eventually::aggregate;

use crate::domain::LightSwitch;
pub type LightSwitchRepo<S> = aggregate::EventSourcedRepository<LightSwitch, S>;

#[derive(Clone)]
pub struct LightSwitchService<R>
where
    R: aggregate::Repository<LightSwitch>,
{
    pub light_switch_repository: R,
}

impl<R> From<R> for LightSwitchService<R>
where
    R: aggregate::Repository<LightSwitch>,
{
    fn from(light_switch_repository: R) -> Self {
        Self {
            light_switch_repository,
        }
    }
}
