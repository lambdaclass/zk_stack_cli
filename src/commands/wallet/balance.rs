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
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let wallet_address = cfg.wallet.context("Wallet config missing")?.address;
    if let Some(token_address) = args.token_address {
        let erc20 = ERC20::new(token_address, Arc::new(provider));
        let balance = erc20.balance_of(wallet_address).await.unwrap();
        let token_symbol = erc20.symbol().await.unwrap();
        println!("Balance: {balance} {token_symbol}");
    } else {
        let balance = provider.get_balance(wallet_address, None).await.unwrap();
        println!("Base Token Balance: {balance}");
    }
    Ok(())
}
