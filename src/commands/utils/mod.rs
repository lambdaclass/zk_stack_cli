use std::sync::Arc;

use eyre::ContextCompat;
use zksync_ethers_rs::{
    contracts::governance::Governance,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};

use crate::config::ZKSyncConfig;

pub(crate) mod balance;

// pub(crate) fn try_l2_provider_from_config(cfg: &ZKSyncConfig) -> eyre::Result<Provider<Http>> {
//     Provider::try_from(cfg.network.l2_rpc_url.as_str()).map_err(Into::into)
// }

pub(crate) fn try_l1_provider_from_config(cfg: &ZKSyncConfig) -> eyre::Result<Provider<Http>> {
    Provider::try_from(
        cfg.network
            .l1_rpc_url
            .as_deref()
            .context("L1 RPC URL not found in config")?,
    )
    .map_err(Into::into)
}

pub(crate) async fn try_governance_from_config(
    cfg: &ZKSyncConfig,
) -> eyre::Result<Governance<SignerMiddleware<impl Middleware, impl Signer>>> {
    let l1_provider = try_l1_provider_from_config(cfg)?;
    let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
    let wallet = cfg
        .governance
        .owner_private_key
        .parse::<LocalWallet>()?
        .with_chain_id(l1_chain_id);
    let l1_signer = SignerMiddleware::new(l1_provider, wallet);
    Ok(Governance::new(cfg.governance.address, Arc::new(l1_signer)))
}
