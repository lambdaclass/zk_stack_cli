use crate::config::ZKSyncConfig;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};

pub(crate) mod balance;
pub(crate) mod config;
pub(crate) mod contracts;
pub(crate) mod db;
pub(crate) mod messages;
pub(crate) mod prover_status;
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
    wallet: LocalWallet,
    cfg: &ZKSyncConfig,
) -> eyre::Result<SignerMiddleware<impl Middleware, impl Signer>> {
    let l1_provider = try_l1_provider_from_config(cfg)?;
    let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
    let wallet = wallet.with_chain_id(l1_chain_id);
    Ok(SignerMiddleware::new(l1_provider, wallet))
}
