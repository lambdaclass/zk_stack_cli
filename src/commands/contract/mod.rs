use clap::Subcommand;

use crate::config::ZKSyncConfig;

pub(crate) mod call;
pub(crate) mod deploy;
pub(crate) mod send;
pub(crate) mod mint;

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "Call view functions on a contract.")]
    Call(call::Args),
    #[clap(about = "Deploy a contract.")]
    Deploy(deploy::Args),
    #[clap(about = "Call non-view functions on a contract.")]
    Send(send::Args),
    #[clap(about = "MINT")]
    Mint(mint::Args),
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    match cmd {
        Command::Call(args) => call::run(args, cfg).await?,
        Command::Deploy(args) => deploy::run(args, cfg).await?,
        Command::Send(args) => send::run(args, cfg).await?,
        Command::Mint(args) => mint::run(args, cfg).await?,
    };

    Ok(())
}
