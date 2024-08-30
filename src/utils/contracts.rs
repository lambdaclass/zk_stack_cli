use crate::{
    config::ZKSyncConfig,
    utils::{try_l1_signer_from_config, try_l2_provider_from_config},
};
use eyre::ContextCompat;
use std::sync::Arc;
use zksync_ethers_rs::{
    contracts::{
        bridgehub::Bridgehub, governance::Governance,
        state_transition_manager::StateTransitionManager,
    },
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

pub(crate) async fn try_state_transition_manager_from_config(
    cfg: &ZKSyncConfig,
) -> eyre::Result<StateTransitionManager<SignerMiddleware<impl Middleware, impl Signer>>> {
    let chain_id = cfg
        .network
        .l2_chain_id
        .ok_or(eyre::eyre!("L2 chain id not found in config"))?;
    let bridgehub = try_bridgehub_from_config(cfg).await?;
    let stm_address = bridgehub
        .state_transition_manager(chain_id.into())
        .call()
        .await?;
    let l1_signer =
        try_l1_signer_from_config(cfg.governance.owner_private_key.parse()?, cfg).await?;

    Ok(StateTransitionManager::new(
        stm_address,
        Arc::new(l1_signer),
    ))
}
