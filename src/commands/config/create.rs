use crate::commands::config::common::{
    config_path, confirm, messages::CONFIG_OVERRIDE_PROMPT_MSG, prompt_zksync_config,
};
use clap::Args as ClapArgs;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "name")]
    pub config_name: String,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let config_path = config_path(&args.config_name)?;
    if config_path.exists() {
        let override_confirmation = confirm(CONFIG_OVERRIDE_PROMPT_MSG)?;
        if !override_confirmation {
            println!("Aborted");
            return Ok(());
        }
    }
    let config = prompt_zksync_config()?;
    let toml_config = toml::to_string_pretty(&config)?;
    println!(
        "Config created at: {}\n{toml_config}",
        config_path.display()
    );
    std::fs::write(config_path, toml_config)?;
    Ok(())
}
