use crate::commands::config::common::{
    config_path, config_path_interactive_selection, confirm_config_creation,
    CONFIG_TO_DISPLAY_PROMPT_MSG,
};
use clap::Args as ClapArgs;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "name", required_unless_present = "select_config_interactively")]
    pub config_name: String,
    #[clap(short, long = "interactively", required_unless_present = "config_name")]
    pub select_config_interactively: bool,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let config_to_display_path = if args.select_config_interactively {
        config_path_interactive_selection(CONFIG_TO_DISPLAY_PROMPT_MSG)?
    } else {
        config_path(&args.config_name)?
    };
    if !config_to_display_path.exists() {
        return confirm_config_creation(args.config_name).await;
    }
    println!("Config at: {}", config_to_display_path.display());
    println!();
    println!("{}", std::fs::read_to_string(config_to_display_path)?);
    Ok(())
}
