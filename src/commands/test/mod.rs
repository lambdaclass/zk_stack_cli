use clap::Subcommand;

use crate::config::ZKSyncConfig;

pub(crate) mod erc20_l1_mint;
pub(crate) mod loadtest;

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "LoadTest the zkStack Chain.")]
    Loadtest(loadtest::Args),
    #[clap(about = "Mint ERC20 token on L1.")]
    Erc20L1Mint(erc20_l1_mint::Args),
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    match cmd {
        Command::Loadtest(args) => loadtest::run(args, cfg).await?,
        Command::Erc20L1Mint(args) => erc20_l1_mint::run(args, cfg).await?,
    };

    Ok(())
}
