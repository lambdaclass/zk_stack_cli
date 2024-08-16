use crate::commands::utils::balance::*;
use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::{providers::Provider, types::Address, ZKMiddleware};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "of", required = true)]
    of: Address,
    #[clap(long = "token")]
    token_address: Option<Address>,
    #[clap(long = "l2", required = false)]
    l2: bool,
    #[clap(long = "l1", required = false)]
    l1: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let wallet_address = args.of;
    let base_token_address = l2_provider.get_base_token_l1_address().await?;

    if args.l2 || !args.l1 {
        println!("Base Token Address: {base_token_address:?}");
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
