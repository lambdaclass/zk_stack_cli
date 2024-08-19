use crate::commands::utils::balance::display_balance;
use crate::commands::utils::wallet::*;
use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use colored::*;
use eyre::ContextCompat;
use std::ops::Div;
use zksync_ethers_rs::{
    core::{k256::ecdsa::SigningKey, rand::thread_rng, utils::parse_ether},
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::U256,
    wait_for_finalize_withdrawal, ZKMiddleware,
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
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

    // ideally it should be the amount transferred, the gas + fees have to be deducted automatically
    // an extra 20% is used to avoid gas problems
    let amount_of_bt_to_deposit: f32 = args.amount * 1.2;
    let float_wallets: f32 = args.number_of_wallets.into();
    let amount_of_bt_to_transfer_for_each: f32 = args.amount / float_wallets;
    let amount_of_bt_to_withdraw: f32 = args.amount;

    // Begin Display L1 Balance and BaseToken Addr
    println!("{}", "#".repeat(64));
    println!(
        "{}: {base_token_address:?}",
        "Base Token Address".bold().green().on_black()
    );
    display_balance(None, &zk_wallet, true).await?;
    display_balance(Some(base_token_address), &zk_wallet, true).await?;
    println!("{}", "#".repeat(64));
    // End Display L1 Balance and BaseToken Addr

    // Begin Deposit from rich wallet to rich wallet
    //println!("{}", "#".repeat(64));
    //println!(
    //    "{} Deposit from {} wallet to {} wallet.",
    //    "[L1->L2]".bold().bright_cyan().on_black(),
    //    "rich".bold().red().on_black(),
    //    "rich".bold().red().on_black()
    //);
    //display_balance(None, &zk_wallet, false).await?;
    //zk_wallet
    //    .deposit_base_token(parse_ether(amount_of_bt_to_deposit.to_string())?)
    //    .await?;
    //display_balance(None, &zk_wallet, false).await?;
    //println!("{}", "#".repeat(64));
    // End Deposit from rich wallet to rich wallet

    // Begin Transfer from rich wallet to each wallet
    println!("{}", "#".repeat(64));
    println!(
        "{} Transfer from {} wallet to {} wallet.",
        "[L2->L2]".bold().bright_cyan().on_black(),
        "rich".bold().red().on_black(),
        "each".bold().blue().on_black()
    );
    for w in &wallets {
        display_balance(None, w, false).await?;
        let transfer_hash = zk_wallet
            .transfer_base_token(
                parse_ether("1")?,
                w.l1_address(),
            )
            .await?;
        println!("Transfer hash: {transfer_hash:?}");
        display_balance(None, w, false).await?;
    }
    println!("{}", "#".repeat(64));
    // End Transfer from rich wallet to each wallet

    // Begin Transfer from each wallet to rich wallet
    println!("{}", "#".repeat(64));
    println!(
        "{} Transfer from {} wallet to {} wallet.",
        "[L1->L2]".bold().bright_cyan().on_black(),
        "each".bold().blue().on_black(),
        "rich".bold().red().on_black()
    );
    for w in &wallets {
        display_balance(None, w, false).await?;
        display_balance(None, &zk_wallet, false).await?;
        let balance = w.l2_provider().get_balance(w.l2_address(), None).await?;
        let transfer_hash = w
            .transfer_base_token(balance, zk_wallet.l1_address())
            .await?;
        println!("Transfer hash: {transfer_hash:?}");
        display_balance(None, w, false).await?;
        display_balance(None, &zk_wallet, false).await?;
    }
    println!("{}", "#".repeat(64));
    // End Transfer from each wallet to rich wallet

    // Begin Withdrawal
    println!("{}", "#".repeat(64));
    println!(
        "{} Withdraw basetoken from {} wallet.",
        "[L2->L1]".bold().bright_cyan().on_black(),
        "rich".bold().red().on_black(),
    );
    display_balance(None, &zk_wallet, false).await?;
    let withdraw_hash = zk_wallet
        .withdraw_base_token(parse_ether(amount_of_bt_to_withdraw.to_string())?)
        .await?;
    println!("Withdraw hash: {withdraw_hash:?}");
    display_balance(None, &zk_wallet, false).await?;
    let base_token_address = Some(l2_provider.get_base_token_l1_address().await?);
    display_balance(base_token_address, &zk_wallet, true).await?;
    println!("finalize withdrawal");
    wait_for_finalize_withdrawal(withdraw_hash, &l2_provider).await;
    zk_wallet.finalize_withdraw(withdraw_hash).await?;
    display_balance(base_token_address, &zk_wallet, true).await?;
    println!("{}", "#".repeat(64));
    // End Withdrawal

    Ok(())
}
