use crate::commands::config::common::{
    config_path, config_path_interactive_selection, confirm,
    messages::{CONFIG_DELETE_PROMPT_MSG, CONFIG_SELECTION_TO_DELETE_PROMPT_MSG},
};
use clap::Args as ClapArgs;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "name")]
    pub config_name: Option<String>,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let config_path = if let Some(config_name) = args.config_name {
        config_path(&config_name)?
    } else {
        config_path_interactive_selection(CONFIG_SELECTION_TO_DELETE_PROMPT_MSG)?
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
