use crate::commands::utils::balance::display_balance;
use crate::commands::utils::wallet::*;
use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use std::ops::Div;
use zksync_ethers_rs::{
    core::{k256::ecdsa::SigningKey, rand::thread_rng, utils::parse_ether},
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    wait_for_finalize_withdrawal, ZKMiddleware,
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "placeholder", required = false)]
    pub placeholder: bool,
    #[clap(long = "wallets", required = true)]
    pub number_of_wallets: u16,
    #[clap(long = "amount", required = true)]
    pub amount: f32,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let wallet_config = cfg.wallet.context("Wallet config missing")?;
    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;

    let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
    let l2_chain_id = l2_provider.get_chainid().await?.as_u64();

    let wallet = wallet_config
        .private_key
        .parse::<Wallet<SigningKey>>()?
        .with_chain_id(l1_chain_id)
        .with_chain_id(l2_chain_id); // is this ok?

    let zk_wallet = new_zkwallet(wallet, &l1_provider, &l2_provider).await?;

    let mut wallets = Vec::new();

    println!("{}", args.placeholder);

    for i in 1..=args.number_of_wallets {
        let local_wallet = LocalWallet::new(&mut thread_rng());
        let pk_bytes = local_wallet.signer().to_bytes();
        //let sk: SigningKey = SigningKey::from_bytes(&pk_bytes)?;
        let pk = hex::encode(pk_bytes);
        println!(
            "Wallet [{i:0>3}] addr: {:?} || pk: 0x{pk}",
            local_wallet.address(),
        );
        let w = new_zkwallet(local_wallet, &l1_provider, &l2_provider).await?;
        wallets.push(w);
    }

    let base_token_address = l2_provider.get_base_token_l1_address().await?;
    let float_number_of_wallets: f32 = args.number_of_wallets.into();
    let amount_to_deposit_for_each: f32 = args.amount.div(float_number_of_wallets);
    let amount_to_transfer_for_each: f32 = amount_to_deposit_for_each.div(2.0);
    // ideally it should be the amount transferred, the gas + fees have to be deducted automatically
    let amount_to_withdraw_for_each: f32 = amount_to_transfer_for_each.div(2.0);

    println!("Base Token Address: {base_token_address:?}");

    for w in &wallets {
        display_balance(None, w, false).await?;
        let deposit_hash = zk_wallet
            .deposit_base_token_to(
                parse_ether(amount_to_deposit_for_each.to_string())?,
                w.l2_address(),
            )
            .await?;
        println!("Deposit hash: {deposit_hash:?}");
        display_balance(None, w, false).await?;
    }

    for w in &wallets {
        display_balance(None, w, false).await?;
        display_balance(None, &zk_wallet, false).await?;
        let transfer_hash = w
            .transfer_base_token(
                parse_ether(amount_to_transfer_for_each.to_string())?,
                zk_wallet.l1_address(),
            )
            .await?;
        println!("Transfer hash: {transfer_hash:?}");
        display_balance(None, w, false).await?;
        display_balance(None, &zk_wallet, false).await?;
    }

    for w in &wallets {
        display_balance(None, w, false).await?;
        display_balance(None, &zk_wallet, false).await?;
        let withdraw_hash = zk_wallet
            .withdraw_base_token(parse_ether(amount_to_withdraw_for_each.to_string())?)
            .await?;
        println!("Withdraw hash: {withdraw_hash:?}");
        display_balance(None, w, false).await?;
        display_balance(None, &zk_wallet, false).await?;
        let base_token_address = Some(l2_provider.get_base_token_l1_address().await?);
        display_balance(base_token_address, &zk_wallet, true).await?;
        println!("finalize withdrawal");
        zk_wallet.finalize_withdraw(withdraw_hash).await?;
        display_balance(base_token_address, &zk_wallet, true).await?;
    }

    display_balance(None, &zk_wallet, false).await?;
    let withdraw_hash = zk_wallet
        .withdraw_base_token(parse_ether(amount_to_withdraw_for_each.to_string())?)
        .await?;
    println!("Withdraw hash: {withdraw_hash:?}");
    display_balance(None, &zk_wallet, false).await?;
    let base_token_address = Some(l2_provider.get_base_token_l1_address().await?);
    display_balance(base_token_address, &zk_wallet, true).await?;
    println!("finalize withdrawal");
    wait_for_finalize_withdrawal(withdraw_hash, &l2_provider).await;
    zk_wallet.finalize_withdraw(withdraw_hash).await?;
    display_balance(base_token_address, &zk_wallet, true).await?;

    Ok(())
}
