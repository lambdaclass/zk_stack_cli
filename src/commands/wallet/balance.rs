use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use std::sync::Arc;
use zksync_ethers_rs::{
    contracts::erc20::ERC20,
    providers::{Middleware, Provider},
    types::Address,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "token")]
    pub token_address: Option<Address>,
    #[clap(long = "l1", required = false)]
    pub l1: bool,
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
    let wallet_address = cfg.wallet.context("Wallet config missing")?.address;
    let network = args.l1.then_some("L1").unwrap_or("L2");
    if let Some(token_address) = args.token_address {
        let erc20 = ERC20::new(token_address, Arc::new(provider));
        let balance = erc20.balance_of(wallet_address).await?;
        let token_symbol = erc20.symbol().await?;
        println!("[{network}] Balance: {balance} {token_symbol}");
    } else {
        let balance = provider.get_balance(wallet_address, None).await?;
        println!("[{network}] Base Token Balance: {balance}");
    }
    Ok(())
}
