use crate::commands::config::common::{config_path, confirm_config_creation};
use clap::Args as ClapArgs;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "name")]
    pub config_name: String,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let config_path = config_path(&args.config_name)?;
    if !config_path.exists() {
        return confirm_config_creation(args.config_name).await;
    }
    println!("Config at: {}", config_path.display());
    println!();
    println!("{}", std::fs::read_to_string(config_path)?);
    Ok(())
}
