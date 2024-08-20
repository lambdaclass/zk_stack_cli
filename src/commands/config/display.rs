use crate::commands::config::common::{
    config_path, config_path_interactive_selection, confirm_config_creation,
    messages::CONFIG_TO_DISPLAY_PROMPT_MSG,
};
use clap::Args as ClapArgs;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "name")]
    pub config_name: Option<String>,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let config_to_display_path = if let Some(config_name) = args.config_name {
        let config_to_display_path = config_path(&config_name)?;
        if !config_to_display_path.exists() {
            return confirm_config_creation(config_name).await;
        }
        config_to_display_path
    } else {
        config_path_interactive_selection(CONFIG_TO_DISPLAY_PROMPT_MSG)?
    };
    println!("Config at: {}", config_to_display_path.display());
    println!();
    println!("{}", std::fs::read_to_string(config_to_display_path)?);
    Ok(())
}
