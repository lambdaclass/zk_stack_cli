use clap::Subcommand;

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod display;
pub(crate) mod edit;
pub(crate) mod list;
pub(crate) mod set;

pub(crate) mod common;

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "Edit an existing config.")]
    Edit(edit::Args),
    #[clap(about = "Create a new config.")]
    Create(create::Args),
    #[clap(about = "Set the config to use.")]
    Set(set::Args),
    #[clap(about = "Display a config.")]
    Display(display::Args),
    #[clap(about = "List all configs.")]
    List,
    #[clap(about = "Delete a config.")]
    Delete(delete::Args),
}

pub(crate) async fn start(cmd: Command) -> eyre::Result<()> {
    match cmd {
        Command::Edit(args) => edit::run(args).await?,
        Command::Create(args) => create::run(args).await?,
        Command::Set(args) => set::run(args).await?,
        Command::Display(args) => display::run(args).await?,
        Command::List => list::run().await?,
        Command::Delete(args) => delete::run(args).await?,
    };

    Ok(())
}
