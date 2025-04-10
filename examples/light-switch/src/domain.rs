use eventually::{aggregate, message};

pub type LightSwitchId = String;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LightSwitchError {
    #[error("Light switch has not yet been installed")]
    NotYetInstalled,
    #[error("Light switch has already been installed")]
    AlreadyInstalled,
    #[error("Light switch is already on")]
    AlreadyOn,
    #[error("Light switch is already off")]
    AlreadyOff,
}

// events
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Installed {
    id: LightSwitchId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SwitchedOn {
    id: LightSwitchId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SwitchedOff {
    id: LightSwitchId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LightSwitchEvent {
    Installed(Installed),
    SwitchedOn(SwitchedOn),
    SwitchedOff(SwitchedOff),
}

impl message::Message for LightSwitchEvent {
    fn name(&self) -> &'static str {
        match self {
            LightSwitchEvent::SwitchedOn(_) => "SwitchedOn",
            LightSwitchEvent::SwitchedOff(_) => "SwitchedOff",
            LightSwitchEvent::Installed(_) => "Installed",
        }
    }
}

// aggregate
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LightSwitchState {
    On,
    Off,
}

#[derive(Debug, Clone)]
pub struct LightSwitch {
    id: LightSwitchId,
    state: LightSwitchState,
}

impl aggregate::Aggregate for LightSwitch {
    type Id = LightSwitchId;
    type Event = LightSwitchEvent;
    type Error = LightSwitchError;

    fn type_name() -> &'static str {
        "LightSwitch"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match state {
            None => match event {
                LightSwitchEvent::Installed(installed) => Ok(LightSwitch {
                    id: installed.id,
                    state: LightSwitchState::Off,
                }),
                LightSwitchEvent::SwitchedOn(_) | LightSwitchEvent::SwitchedOff(_) => {
                    Err(LightSwitchError::NotYetInstalled)
                },
            },
            Some(mut light_switch) => match event {
                LightSwitchEvent::Installed(_) => Err(LightSwitchError::AlreadyInstalled),
                LightSwitchEvent::SwitchedOn(_) => match light_switch.state {
                    LightSwitchState::On => Err(LightSwitchError::AlreadyOn),
                    LightSwitchState::Off => {
                        light_switch.state = LightSwitchState::On;
                        Ok(light_switch)
                    },
                },
                LightSwitchEvent::SwitchedOff(_) => match light_switch.state {
                    LightSwitchState::On => {
                        light_switch.state = LightSwitchState::Off;
                        Ok(light_switch)
                    },
                    LightSwitchState::Off => Err(LightSwitchError::AlreadyOff),
                },
            },
        }
    }
}

// root
#[derive(Debug, Clone)]
pub struct LightSwitchRoot(aggregate::Root<LightSwitch>);

// NOTE: The trait implementations for From, Deref and DerefMut below are
// implemented manually for demonstration purposes, but most would prefer to have them
// auto-generated at compile time by using the [`eventually_macros::aggregate_root`] macro
impl From<eventually::aggregate::Root<LightSwitch>> for LightSwitchRoot {
    fn from(root: eventually::aggregate::Root<LightSwitch>) -> Self {
        Self(root)
    }
}
impl From<LightSwitchRoot> for eventually::aggregate::Root<LightSwitch> {
    fn from(value: LightSwitchRoot) -> Self {
        value.0
    }
}
impl std::ops::Deref for LightSwitchRoot {
    type Target = eventually::aggregate::Root<LightSwitch>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for LightSwitchRoot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl LightSwitchRoot {
    pub fn install(id: LightSwitchId) -> Result<Self, LightSwitchError> {
        aggregate::Root::<LightSwitch>::record_new(
            LightSwitchEvent::Installed(Installed { id }).into(),
        )
        .map(Self)
    }
    pub fn turn_on(&mut self, id: LightSwitchId) -> Result<(), LightSwitchError> {
        self.record_that(LightSwitchEvent::SwitchedOn(SwitchedOn { id }).into())
    }
    pub fn turn_off(&mut self, id: LightSwitchId) -> Result<(), LightSwitchError> {
        self.record_that(LightSwitchEvent::SwitchedOff(SwitchedOff { id }).into())
    }
    pub fn get_switch_state(&self) -> Result<LightSwitchState, LightSwitchError> {
        Ok(self.state.clone())
    }
}
