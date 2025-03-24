use eventually::aggregate;

use crate::domain::Lightswitch;
pub type LightswitchRepo<S> = aggregate::EventSourcedRepository<Lightswitch, S>;

#[derive(Clone)]
pub struct LightswitchService<R>
where
    R: aggregate::Repository<Lightswitch>,
{
    pub light_switch_repository: R,
}

impl<R> From<R> for LightswitchService<R>
where
    R: aggregate::Repository<Lightswitch>,
{
    fn from(light_switch_repository: R) -> Self {
        Self {
            light_switch_repository,
        }
    }
}
