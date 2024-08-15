use crate::commands::utils::wallet;
use crate::commands::utils::wallet::*;
use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use std::str::FromStr;
use zksync_ethers_rs::core::rand::thread_rng;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::signers::LocalWallet;
use zksync_ethers_rs::signers::Signer;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "placeholder", required = false)]
    pub placeholder: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let zk_wallet = wallet::new_zkwallet(
        LocalWallet::from_str(
            "0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8",
        )?,
        &l1_provider,
        &l2_provider,
    )
    .await?;

    let mut wallets = Vec::new();

    println!("{}", args.placeholder);

    for i in 1_i32..=100_i32 {
        let local_wallet = LocalWallet::new(&mut thread_rng());
        let pk_bytes = local_wallet.signer().to_bytes();
        //let sk: SigningKey = SigningKey::from_bytes(&pk_bytes)?;
        let pk = hex::encode(pk_bytes);
        println!("Wallet {i:0>3}: {:?} || 0x{pk}", local_wallet.address(),);
        let w = new_zkwallet(local_wallet, &l1_provider, &l2_provider).await?;
        wallets.push(w);
    }

    println!("{}", zk_wallet.l1_address());
    //zk_wallet.deposit_base_token(parse_ether("1")?).await?;

    Ok(())
}
