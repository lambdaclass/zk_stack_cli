use crate::{commands::utils::try_l1_signer_from_config, config::ZKSyncConfig};
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use std::sync::Arc;
use zksync_ethers_rs::{contracts::bridgehub::Bridgehub, providers::Middleware, types::Address};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(short, long = "Address")]
    pub new_pending_admin: Address,
    #[clap(short, long)]
    pub pending_admin_private_key: String,
}

pub(crate) async fn run(
    args: Args,
    bridgehub: Bridgehub<impl Middleware + 'static>,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    // TODO: Do not repeat the same code for calling the set_pending_admin method.
    let transaction_receipt = if let Some(ref admin_private_key) = cfg.bridgehub.admin_private_key {
        let current_admin = try_l1_signer_from_config(admin_private_key, &cfg).await?;
        // We need to instantiate a Bridgehub with the pending admin as the signer
        // to be able to call accept_admin
        let bridgehub = Bridgehub::new(bridgehub.address(), Arc::new(current_admin));
        bridgehub
            .set_pending_admin(args.new_pending_admin)
            .send()
            .await?
            .await?
            .context("No transaction receipt for bridgehub admin acceptance")?
    } else {
        bridgehub
            .set_pending_admin(args.new_pending_admin)
            .send()
            .await?
            .await?
            .context("No transaction receipt for bridgehub admin acceptance")?
    };
    println!(
        "New Bridgehub pending admin: {:?}",
        transaction_receipt.transaction_hash
    );
    Ok(())
}
