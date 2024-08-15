use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use std::str::FromStr;
use zksync_ethers_rs::{
    abi::Hash, finalize_withdrawal, middleware::SignerMiddleware, providers::Provider,
    signers::LocalWallet, types::Address,
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "hash")]
    pub l2_withdraw_tx_hash: Hash,
    #[clap(long = "to")]
    pub to: Option<Address>,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let l1_provider = Provider::try_from(cfg.network.l1_rpc_url.context("L1 RPC URL is needed")?)?;
    let wallet = LocalWallet::from_str(&cfg.wallet.context("Wallet config missing")?.private_key)?;
    let signer = SignerMiddleware::new(l1_provider, wallet);
    finalize_withdrawal(signer.into(), args.l2_withdraw_tx_hash, &l2_provider).await;
    Ok(())
}
