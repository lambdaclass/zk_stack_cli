use crate::commands::utils::erc20::erc20_mint;
use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::core::k256::ecdsa::SigningKey;
use zksync_ethers_rs::core::utils::parse_ether;
use zksync_ethers_rs::middleware::SignerMiddleware;
use zksync_ethers_rs::providers::Middleware;
use zksync_ethers_rs::signers::{Signer, Wallet};
use zksync_ethers_rs::zk_wallet::ZKWallet;
use zksync_ethers_rs::{providers::Provider, types::Address, ZKMiddleware};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "token")]
    pub token_address: Option<Address>,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let wallet_address = cfg.wallet.context("Wallet config missing")?.address;
    let base_token_address = l2_provider.get_base_token_l1_address().await?;

    let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
    let l2_chain_id = l2_provider.get_chainid().await?.as_u64();

    let wallet = cfg
        .wallet
        .context("Wallet config missing")?
        .private_key
        .parse::<Wallet<SigningKey>>()?
        .with_chain_id(l1_chain_id)
        .with_chain_id(l2_chain_id);

    let l1_signer = SignerMiddleware::new(l1_provider, wallet.clone());
    let l2_signer = SignerMiddleware::new(l2_provider, wallet);

    let zk_wallet = ZKWallet::new(l1_signer, l2_signer);

    erc20_mint(base_token_address, zk_wallet, parse_ether("10")?, false).await?;
    Ok(())
}
