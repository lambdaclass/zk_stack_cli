use crate::commands::config::common::{
    config_path, config_path_interactive_selection, confirm, CONFIG_DELETE_PROMPT_MSG,
};
use clap::Args as ClapArgs;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(
        long = "name",
        conflicts_with = "delete_interactively",
        required_unless_present = "delete_interactively"
    )]
    pub config_name: Option<String>,
    #[clap(
        short,
        long = "interactively",
        conflicts_with = "config_name",
        required_unless_present = "config_name"
    )]
    pub delete_interactively: bool,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let config_path = if args.delete_interactively {
        config_path_interactive_selection(CONFIG_DELETE_PROMPT_MSG)?
    } else {
        config_path(&args.config_name.unwrap())?
    };
    let delete_confirmation = confirm(CONFIG_DELETE_PROMPT_MSG)?;
    if !delete_confirmation {
        println!("Aborted");
        return Ok(());
    }
    std::fs::remove_file(config_path.clone())?;
    println!("Removed config at: {}", config_path.display());
    Ok(())
}
