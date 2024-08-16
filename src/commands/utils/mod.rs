use crate::config::ZKSyncConfig;
use std::sync::Arc;
use zksync_ethers_rs::{
    contracts::governance::Governance,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};

pub(crate) mod balance;

pub(crate) fn try_l2_provider_from_config(cfg: &ZKSyncConfig) -> eyre::Result<Provider<Http>> {
    Provider::try_from(cfg.network.l2_rpc_url.as_str()).map_err(Into::into)
}

pub(crate) async fn try_governance_from_config(
    cfg: &ZKSyncConfig,
) -> eyre::Result<Governance<SignerMiddleware<impl Middleware, impl Signer>>> {
    let l2_provider = try_l2_provider_from_config(&cfg)?;
    let l2_chain_id = l2_provider.get_chainid().await?.as_u64();
    let wallet = cfg
        .governance
        .owner_private_key
        .parse::<LocalWallet>()?
        .with_chain_id(l2_chain_id);
    let l2_signer = SignerMiddleware::new(l2_provider, wallet);
    Ok(Governance::new(cfg.governance.address, Arc::new(l2_signer)))
}
