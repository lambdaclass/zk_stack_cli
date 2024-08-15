use crate::commands::utils::balance::*;
use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::{providers::Provider, types::Address, ZKMiddleware};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "token")]
    pub token_address: Option<Address>,
    #[clap(long = "l2", required = false)]
    pub l2: bool,
    #[clap(long = "l1", required = false)]
    pub l1: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let wallet_address = cfg.wallet.context("Wallet config missing")?.address;
    let base_token_address = l2_provider.get_base_token_l1_address().await?;

    if args.l2 || !args.l1 {
        display_l2_balance(
            args.token_address,
            &l1_provider,
            &l2_provider,
            wallet_address,
            base_token_address,
            args.l1,
        )
        .await?;
    };
    if args.l1 {
        display_l1_balance(args.token_address, &l1_provider, wallet_address).await?;
    };

    Ok(())
}
