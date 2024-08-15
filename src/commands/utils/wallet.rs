use zksync_ethers_rs::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    zk_wallet::ZKWallet,
};

pub(crate) async fn new_zkwallet<'providers>(
    wallet_pk: LocalWallet,
    l1_provider: &'providers Provider<Http>,
    l2_provider: &'providers Provider<Http>,
) -> eyre::Result<ZKWallet<&'providers Provider<Http>, Wallet<SigningKey>>> {
    let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
    let wallet = wallet_pk.with_chain_id(l1_chain_id);
    let l1_signer = SignerMiddleware::new(l1_provider, wallet.clone());

    let l2_chain_id = l2_provider.get_chainid().await?.as_u64();
    let wallet = wallet.with_chain_id(l2_chain_id);
    let l2_signer = SignerMiddleware::new(l2_provider, wallet);

    let zk_wallet = ZKWallet::new(l1_signer, l2_signer);
    Ok(zk_wallet)
}
