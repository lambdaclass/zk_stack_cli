use crate::config::ZKSyncConfig;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    zk_wallet::ZKWallet,
};

use super::{try_l1_provider_from_config, try_l2_provider_from_config};

type ZKWalletProvider = Provider<Http>;
type SetupResult = (
    ZKWallet<ZKWalletProvider, LocalWallet>,
    ZKWalletProvider,
    ZKWalletProvider,
);

impl TryFrom<&ZKSyncConfig> for ZKWallet<ZKWalletProvider, LocalWallet> {
    type Error = eyre::Error;

    fn try_from(cfg: &ZKSyncConfig) -> eyre::Result<Self> {
        let l1_provider = try_l1_provider_from_config(cfg)?;
        let l2_provider = try_l2_provider_from_config(cfg)?;

        let l1_chain_id = cfg
            .network
            .l1_chain_id
            .context("L1 CHAIN_ID missing in config")?;

        let l2_chain_id = cfg
            .network
            .l2_chain_id
            .context("L2 CHAIN_ID missing in config")?;

        let wallet = cfg
            .wallet
            .clone()
            .context("Wallet config missing")?
            .private_key
            .parse::<LocalWallet>()?;

        let wallet = wallet.with_chain_id(l1_chain_id);
        let l1_signer = SignerMiddleware::new(l1_provider.clone(), wallet.clone());

        let wallet = wallet.with_chain_id(l2_chain_id);
        let l2_signer = SignerMiddleware::new(l2_provider.clone(), wallet);

        let zk_wallet = ZKWallet::new(l1_signer.into(), l2_signer.into());

        Ok(zk_wallet)
    }
}

pub(crate) async fn new_zkwallet(
    wallet_pk: LocalWallet,
    l1_provider: &ZKWalletProvider,
    l2_provider: &ZKWalletProvider,
) -> eyre::Result<ZKWallet<ZKWalletProvider, LocalWallet>> {
    let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
    let wallet = wallet_pk.with_chain_id(l1_chain_id);
    let l1_signer = SignerMiddleware::new(l1_provider.clone(), wallet.clone());

    let l2_chain_id = l2_provider.get_chainid().await?.as_u64();
    let wallet = wallet.with_chain_id(l2_chain_id);
    let l2_signer = SignerMiddleware::new(l2_provider.clone(), wallet);

    let zk_wallet = ZKWallet::new(l1_signer.into(), l2_signer.into());
    Ok(zk_wallet)
}

pub fn get_wallet_l1_l2_providers(cfg: ZKSyncConfig) -> eyre::Result<SetupResult> {
    let l1_provider = try_l1_provider_from_config(&cfg)?;
    let l2_provider = try_l2_provider_from_config(&cfg)?;
    let zk_wallet = ZKWallet::try_from(&cfg)?;

    Ok((zk_wallet, l1_provider, l2_provider))
}
