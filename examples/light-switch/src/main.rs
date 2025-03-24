mod application;
mod commands;
mod domain;
mod queries;
use application::{LightswitchRepo, LightswitchService};
use commands::install_light_switch::InstallLightSwitch;
use commands::turn_light_switch_off::TurnLightSwitchOff;
use commands::turn_light_switch_on::TurnLightSwitchOn;
use domain::{LightswitchEvent, LightswitchId};
use eventually::{command, event, query};
use queries::get_switch_state::GetSwitchState;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let store = event::store::InMemory::<LightswitchId, LightswitchEvent>::default();
    let repo = LightswitchRepo::from(store.clone());
    let svc = LightswitchService::from(repo);

    let cmd = InstallLightSwitch {
        id: "Switch1".to_string(),
    }
    .into();
    command::Handler::handle(&svc, cmd).await?;
    println!("Installed Switch1");

    let cmd = TurnLightSwitchOn {
        id: "Switch1".to_string(),
    }
    .into();
    command::Handler::handle(&svc, cmd).await?;
    println!("Turned Switch1 On");

    let cmd = TurnLightSwitchOff {
        id: "Switch1".to_string(),
    }
    .into();
    command::Handler::handle(&svc, cmd).await?;
    println!("Turned Switch1 Off");

    let query = GetSwitchState {
        id: "Switch1".to_string(),
    }
    .into();
    let state = query::Handler::handle(&svc, query).await?;
    println!("Switch1 is currently: {:?}", state);
    Ok(())
}
