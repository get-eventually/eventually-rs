use envconfig::Envconfig;

use eventually_test::config::Config;

fn main() -> anyhow::Result<()> {
    let config = Config::init()?;
    smol::run(eventually_test::run(config))
}
