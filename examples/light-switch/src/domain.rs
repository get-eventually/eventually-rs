use eventually::{aggregate, message};
use eventually_macros::aggregate_root;

pub type LightswitchId = String;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LightswitchError {
    #[error("NotImplemented")]
    NotImplemented,
    #[error("Light switch is already on")]
    AlreadyOn,
    #[error("Light switch is already off")]
    AlreadyOff,
}

// events
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Installed {
    id: LightswitchId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SwitchedOn {
    id: LightswitchId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SwitchedOff {
    id: LightswitchId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LightswitchEvent {
    Installed(Installed),
    SwitchedOn(SwitchedOn),
    SwitchedOff(SwitchedOff),
}

impl message::Message for LightswitchEvent {
    fn name(&self) -> &'static str {
        match self {
            LightswitchEvent::SwitchedOn(_) => "SwitchedOn",
            LightswitchEvent::SwitchedOff(_) => "SwitchedOff",
            LightswitchEvent::Installed(_) => "Installed",
        }
    }
}

// aggregate
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LightswitchState {
    ON,
    OFF,
}

#[derive(Debug, Clone)]
pub struct Lightswitch {
    id: LightswitchId,
    state: LightswitchState,
}

impl aggregate::Aggregate for Lightswitch {
    type Id = LightswitchId;
    type Event = LightswitchEvent;
    type Error = LightswitchError;

    fn type_name() -> &'static str {
        "Lightswitch"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match state {
            None => match event {
                LightswitchEvent::Installed(installed) => Ok(Lightswitch {
                    id: installed.id,
                    state: LightswitchState::OFF,
                }),
                LightswitchEvent::SwitchedOn(_) | LightswitchEvent::SwitchedOff(_) => {
                    Err(LightswitchError::NotImplemented)
                },
            },
            Some(mut light_switch) => match event {
                LightswitchEvent::Installed(_) => Err(LightswitchError::NotImplemented),
                LightswitchEvent::SwitchedOn(_) => match light_switch.state {
                    LightswitchState::ON => Err(LightswitchError::AlreadyOn),
                    LightswitchState::OFF => {
                        light_switch.state = LightswitchState::ON;
                        Ok(light_switch)
                    },
                },
                LightswitchEvent::SwitchedOff(_) => match light_switch.state {
                    LightswitchState::ON => {
                        light_switch.state = LightswitchState::OFF;
                        Ok(light_switch)
                    },
                    LightswitchState::OFF => Err(LightswitchError::AlreadyOff),
                },
            },
        }
    }
}

#[aggregate_root(Lightswitch)]
#[derive(Debug, Clone)]
pub struct LightswitchRoot;

impl LightswitchRoot {
    pub fn install(id: LightswitchId) -> Result<Self, LightswitchError> {
        aggregate::Root::<Lightswitch>::record_new(
            LightswitchEvent::Installed(Installed { id }).into(),
        )
        .map(Self)
    }
    pub fn turn_on(&mut self, id: LightswitchId) -> Result<(), LightswitchError> {
        if self.state == LightswitchState::ON {
            return Err(LightswitchError::AlreadyOn);
        }

        self.record_that(LightswitchEvent::SwitchedOn(SwitchedOn { id }).into())
    }
    pub fn turn_off(&mut self, id: LightswitchId) -> Result<(), LightswitchError> {
        if self.state == LightswitchState::OFF {
            return Err(LightswitchError::AlreadyOff);
        }

        self.record_that(LightswitchEvent::SwitchedOff(SwitchedOff { id }).into())
    }
    pub fn get_switch_state(&self) -> Result<LightswitchState, LightswitchError> {
        Ok(self.state.clone())
    }
}
