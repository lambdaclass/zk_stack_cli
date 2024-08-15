use crate::commands::config::common::{
    config_path, config_path_interactive_selection, confirm_config_creation, selected_config_path,
    CONFIG_SET_PROMPT_MSG,
};
use clap::Args as ClapArgs;
use eyre::ContextCompat;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(
        long = "name",
        conflicts_with = "set_config_interactively",
        required_unless_present = "set_config_interactively"
    )]
    pub config_name: Option<String>,
    #[clap(
        short,
        long = "interactively",
        conflicts_with = "config_name",
        required_unless_present = "config_name"
    )]
    pub set_config_interactively: bool,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let config_path_to_select = if args.set_config_interactively {
        config_path_interactive_selection(CONFIG_SET_PROMPT_MSG)?
    } else {
        let config_name = args.config_name.context("Config name missing")?;
        let config_path_to_select = config_path(&config_name)?;
        if !config_path_to_select.exists() {
            return confirm_config_creation(config_name).await;
        }
        config_path_to_select
    };
    let selected_config = std::fs::read_to_string(config_path_to_select)?;
    std::fs::write(selected_config_path()?, &selected_config)?;
    println!("Config \"{selected_config}\" set");
    Ok(())
}
