#![allow(missing_docs)]

use envconfig::Envconfig;

use eventually_app_example::config::Config;

fn main() -> anyhow::Result<()> {
    let config = Config::init()?;
    smol::run(eventually_app_example::run(config))
}
