use crate::commands::config::common::config_file_names;

pub(crate) async fn run() -> eyre::Result<()> {
    let config_file_names = config_file_names()?;
    if config_file_names.is_empty() {
        println!("No configs found");
        return Ok(());
    }
    println!("Configs:");
    for config_file_name in config_file_names {
        println!("{config_file_name}");
    }
    Ok(())
}
