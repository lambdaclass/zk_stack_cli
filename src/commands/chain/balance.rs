use std::sync::Arc;

use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::contracts::erc20::ERC20;
use zksync_ethers_rs::providers::{Middleware, Provider};
use zksync_ethers_rs::types::{Address, U256};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "of")]
    of: Address,
    #[clap(long = "token")]
    token_address: Option<Address>,
    #[clap(long = "l1", required = false)]
    l1: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = if args.l1 {
        Provider::try_from(
            cfg.network
                .l1_rpc_url
                .context("L1 RPC URL missing in config")?,
        )?
    } else {
        Provider::try_from(cfg.network.l2_rpc_url)?
    };
    let network = if args.l1 { "L1" } else { "L2" };
    if let Some(token_address) = args.token_address {
        let erc20 = ERC20::new(token_address, Arc::new(provider));
        let balance: U256 = erc20.balance_of(args.of).call().await?;
        let token_symbol = erc20.symbol().call().await?;
        println!("[{network}] Balance: {balance} {token_symbol}");
    } else {
        let balance = provider.get_balance(args.of, None).await?;
        println!("[{network}] Balance: {balance}");
    }
    Ok(())
}
