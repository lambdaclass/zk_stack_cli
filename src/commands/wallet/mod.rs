use clap::Subcommand;

use crate::config::ZKSyncConfig;

pub(crate) mod balance;
pub(crate) mod deposit;
pub(crate) mod finalize_withdraw;
pub(crate) mod transfer;
pub(crate) mod withdraw;

#[derive(Subcommand)]
pub(crate) enum Command {
    Balance(balance::Args),
    Deposit(deposit::Args),
    FinalizeWithdraw(finalize_withdraw::Args),
    Transfer(transfer::Args),
    Withdraw,
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    match cmd {
        Command::Balance(args) => balance::run(args, cfg).await?,
        Command::Deposit(args) => deposit::run(args, cfg).await?,
        Command::FinalizeWithdraw(args) => finalize_withdraw::run(args, cfg).await?,
        Command::Transfer(args) => transfer::run(args, cfg).await?,
        Command::Withdraw => todo!("Withdraw"),
    };

    Ok(())
}
