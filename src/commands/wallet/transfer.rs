use std::{str::FromStr, sync::Arc};

use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    contracts::erc20::ERC20,
    middleware::SignerMiddleware,
    providers::Provider,
    signers::LocalWallet,
    types::{Address, U256},
};

use crate::config::ZKSyncConfig;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "amount")]
    pub amount: U256,
    #[clap(long = "token")]
    pub token_address: Option<Address>,
    #[clap(long = "to")]
    pub to: Address,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let wallet = LocalWallet::from_str(&cfg.wallet.context("Wallet config missing")?.private_key)?;
    let signer = SignerMiddleware::new(provider, wallet);
    if let Some(token_address) = args.token_address {
        let erc20 = ERC20::new(token_address, Arc::new(signer));
        erc20
            .transfer(args.to, args.amount)
            .send()
            .await?
            .await?
            .context("Failed to transfer ERC20 token")?;
    } else {
        todo!("Base token transfer");
    }
    todo!()
}
