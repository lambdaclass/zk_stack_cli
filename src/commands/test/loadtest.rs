use std::str::FromStr;

use crate::commands::utils::balance::*;
use crate::commands::utils::wallet::*;
use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::core::utils::parse_ether;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::signers::LocalWallet;
use zksync_ethers_rs::types::Address;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "placeholder", required = false)]
    pub placeholder: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let zk_wallet = new_zkwallet(
        LocalWallet::from_str(
            "0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8",
        )?,
        &l1_provider,
        &l2_provider,
    )
    .await?;

    zk_wallet.deposit_base_token_to(parse_ether("1")?, ).await?;

    Ok(())
}
