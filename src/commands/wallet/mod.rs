use clap::Subcommand;

use crate::config::ZKSyncConfig;

pub(crate) mod address;
pub(crate) mod balance;
pub(crate) mod deposit;
pub(crate) mod finalize_withdraw;
pub(crate) mod private_key;
pub(crate) mod transfer;
pub(crate) mod withdraw;

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "Get the balance of the wallet.")]
    Balance(balance::Args),
    #[clap(about = "Deposit funds into the wallet.")]
    Deposit(deposit::Args),
    #[clap(about = "Finalize a pending withdrawal.")]
    FinalizeWithdraw(finalize_withdraw::Args),
    #[clap(about = "Transfer funds to another wallet.")]
    Transfer(transfer::Args),
    #[clap(about = "Withdraw funds from the wallet.")]
    Withdraw,
    #[clap(about = "Get the wallet address.")]
    Address,
    #[clap(about = "Get the wallet private key.")]
    PrivateKey,
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    match cmd {
        Command::Balance(args) => balance::run(args, cfg).await?,
        Command::Deposit(args) => deposit::run(args, cfg).await?,
        Command::FinalizeWithdraw(args) => finalize_withdraw::run(args, cfg).await?,
        Command::Transfer(args) => transfer::run(args, cfg).await?,
        Command::Withdraw => todo!("Withdraw"),
        Command::Address => address::run(cfg).await?,
        Command::PrivateKey => private_key::run(cfg).await?,
    };

    Ok(())
}
