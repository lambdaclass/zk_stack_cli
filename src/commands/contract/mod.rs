use clap::Subcommand;

use crate::config::ZKSyncConfig;

pub(crate) mod call;
pub(crate) mod deploy;
pub(crate) mod send;

#[derive(Subcommand)]
pub(crate) enum Command {
    Call(call::Args),
    Deploy(deploy::Args),
    Send(send::Args),
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    match cmd {
        Command::Call(args) => call::run(args, cfg).await?,
        Command::Deploy(args) => deploy::run(args, cfg).await?,
        Command::Send(args) => send::run(args, cfg).await?,
    };

    Ok(())
}
