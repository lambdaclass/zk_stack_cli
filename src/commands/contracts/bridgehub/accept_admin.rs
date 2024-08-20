use crate::{commands::utils::try_l1_signer_from_config, config::ZKSyncConfig};
use clap::Parser;
use eyre::ContextCompat;
use std::sync::Arc;
use zksync_ethers_rs::{contracts::bridgehub::Bridgehub, providers::Middleware};

#[derive(Parser, PartialEq)]
pub(crate) struct Args {
    pub pending_admin_private_key: String,
}

pub(crate) async fn run(
    args: Args,
    bridgehub: Bridgehub<impl Middleware + 'static>,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    let pending_admin = try_l1_signer_from_config(&args.pending_admin_private_key, &cfg).await?;
    // We need to instantiate a Bridgehub with the pending admin as the signer
    // to be able to call accept_admin
    let bridgehub = Bridgehub::new(bridgehub.address(), Arc::new(pending_admin));
    let transaction_receipt = bridgehub
        .accept_admin()
        .send()
        .await?
        .await?
        .context("No transaction receipt for bridgehub admin acceptance")?;
    println!(
        "New Bridgehub admin accepted: {:?}",
        transaction_receipt.transaction_hash
    );
    Ok(())
}
