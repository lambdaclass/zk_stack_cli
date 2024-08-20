use crate::{
    utils::{try_l1_signer_from_config, try_l2_provider_from_config},
    config::ZKSyncConfig,
};
use eyre::ContextCompat;
use std::sync::Arc;
use zksync_ethers_rs::{
    contracts::{bridgehub::Bridgehub, governance::Governance},
    middleware::SignerMiddleware,
    providers::Middleware,
    signers::Signer,
    ZKMiddleware,
};

pub(crate) async fn try_governance_from_config(
    cfg: &ZKSyncConfig,
) -> eyre::Result<Governance<SignerMiddleware<impl Middleware, impl Signer>>> {
    let l1_signer =
        try_l1_signer_from_config(cfg.governance.owner_private_key.parse()?, cfg).await?;
    Ok(Governance::new(cfg.governance.address, Arc::new(l1_signer)))
}

pub(crate) async fn try_bridgehub_from_config(
    cfg: &ZKSyncConfig,
) -> eyre::Result<Bridgehub<SignerMiddleware<impl Middleware, impl Signer>>> {
    let bridgehub_owner = cfg
        .bridgehub
        .owner_private_key
        .as_deref()
        .context("Bridgehub owner private key not found in config")?;
    let l1_signer = try_l1_signer_from_config(bridgehub_owner.parse()?, cfg).await?;
    let bridgehub_address = try_l2_provider_from_config(cfg)?
        .get_bridgehub_contract()
        .await?;
    Ok(Bridgehub::new(bridgehub_address, Arc::new(l1_signer)))
}
