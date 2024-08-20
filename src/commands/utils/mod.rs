use std::sync::Arc;

use eyre::ContextCompat;
use zksync_ethers_rs::{
    contracts::{bridgehub::Bridgehub, governance::Governance},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    ZKMiddleware,
};

use crate::config::ZKSyncConfig;

pub(crate) mod balance;
pub(crate) mod wallet;

pub(crate) fn try_l2_provider_from_config(cfg: &ZKSyncConfig) -> eyre::Result<Provider<Http>> {
    Provider::try_from(cfg.network.l2_rpc_url.as_str()).map_err(Into::into)
}

pub(crate) fn try_l1_provider_from_config(cfg: &ZKSyncConfig) -> eyre::Result<Provider<Http>> {
    Provider::try_from(
        cfg.network
            .l1_rpc_url
            .as_deref()
            .context("L1 RPC URL not found in config")?,
    )
    .map_err(Into::into)
}

pub(crate) async fn try_l1_signer_from_config(
    private_key: &str,
    cfg: &ZKSyncConfig,
) -> eyre::Result<SignerMiddleware<impl Middleware, impl Signer>> {
    let l1_provider = try_l1_provider_from_config(cfg)?;
    let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
    let wallet = private_key
        .parse::<LocalWallet>()?
        .with_chain_id(l1_chain_id);
    Ok(SignerMiddleware::new(l1_provider, wallet))
}

pub(crate) async fn try_governance_from_config(
    cfg: &ZKSyncConfig,
) -> eyre::Result<Governance<SignerMiddleware<impl Middleware, impl Signer>>> {
    let l1_signer = try_l1_signer_from_config(&cfg.governance.owner_private_key, cfg).await?;
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
    let l1_signer = try_l1_signer_from_config(bridgehub_owner, cfg).await?;
    let bridgehub_address = try_l2_provider_from_config(cfg)?
        .get_bridgehub_contract()
        .await?;
    Ok(Bridgehub::new(bridgehub_address, Arc::new(l1_signer)))
}
