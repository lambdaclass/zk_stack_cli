use clap::Subcommand;

use crate::config::ZKSyncConfig;

pub(crate) mod loadtest;

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "LoadTest the zkStack Chain.")]
    Loadtest(loadtest::Args),
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    match cmd {
        Command::Loadtest(args) => loadtest::run(args, cfg).await?,
    };

    Ok(())
}
