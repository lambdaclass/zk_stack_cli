use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_web3_rs::{
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
    utils::parse_units,
    zks_wallet::DepositRequest,
    ZKSWallet,
};

#[derive(ClapArgs)]
pub struct Args {
    #[clap(short, long, name = "AMOUNT_TO_DEPOSIT_IN_ETHER")]
    amount: U256,
    #[clap(short, long, name = "SENDER_PRIVATE_KEY")]
    from: LocalWallet,
    #[clap(short, long, name = "RECEIVER_ADDRESS")]
    to: Option<Address>,
    #[clap(short, long, name = "CHAIN_ID")]
    chain_id: u16,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let request = DepositRequest::new(parse_units(args.amount, "ether")?.into());
    log::info!("Amount to deposit: {}", request.amount);

    let l1_provider = if let Some(port) = config.l1_port {
        Provider::try_from(format!("http://{host}:{port}", host = config.host))?
    } else {
        Provider::try_from(config.host.clone())?
    };
    let l2_provider = if let Some(port) = config.l2_port {
        Provider::try_from(format!("http://{host}:{port}", host = config.host))?
    } else {
        Provider::try_from(config.host.clone())?
    };
    let wallet = args.from.with_chain_id(args.chain_id);
    let zk_wallet = ZKSWallet::new(
        wallet,
        None,
        Some(l2_provider.clone()),
        Some(l1_provider.clone()),
    )?;

    let l1_balance_before = zk_wallet.eth_balance().await?;
    let l2_balance_before = if let Some(to) = args.to {
        l2_provider.get_balance(to, None).await?
    } else {
        zk_wallet.era_balance().await?
    };
    println!("L1 balance before: {l1_balance_before}");
    println!("L2 balance before: {l2_balance_before}");

    let tx_hash = zk_wallet.deposit(&request).await?;
    let receipt = l1_provider
        .get_transaction_receipt(tx_hash)
        .await?
        .context("Failed to get transaction receipt")?;

    let _l2_receipt = l2_provider
        .get_transaction_receipt(receipt.transaction_hash)
        .await?;

    let l1_balance_after = zk_wallet.eth_balance().await?;
    let l2_balance_after = if let Some(to) = args.to {
        l2_provider.get_balance(to, None).await?
    } else {
        zk_wallet.era_balance().await?
    };
    log::info!("L1 balance after: {l1_balance_after}");
    log::info!("L2 balance after: {l2_balance_after}");
    Ok(())
}
