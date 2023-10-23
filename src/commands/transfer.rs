use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_web3_rs::{
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
    zks_wallet::TransferRequest,
    ZKSWallet,
};

use crate::cli::ZKSyncConfig;

#[derive(ClapArgs)]
pub struct Args {
    #[clap(long, name = "AMOUNT_TO_TRANSFER")]
    pub amount: U256,
    #[clap(short, long, name = "SENDER_PRIVATE_KEY")]
    pub from: LocalWallet,
    #[clap(long, name = "RECEIVER_ADDRESS")]
    pub to: Address,
    #[clap(long, name = "CHAIN_ID")]
    pub chain_id: u16,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let era_provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.port
    ))?;
    let wallet = args.from.with_chain_id(args.chain_id);
    let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None)?;

    let sender_balance_before = era_provider
        .get_balance(zk_wallet.l2_address(), None)
        .await?;
    let receiver_balance_before = era_provider.get_balance(args.to, None).await?;

    log::info!("Sender balance before: {sender_balance_before}");
    log::info!("Receiver balance before: {receiver_balance_before}");

    let request = TransferRequest::new(args.amount)
        .to(args.to)
        .from(zk_wallet.l2_address());
    let tx_hash = zk_wallet.transfer(&request, None).await?;

    let receipt = era_provider
        .get_transaction_receipt(tx_hash)
        .await?
        .context("Failed to get transaction receipt")?;

    log::info!("Transaction receipt: {receipt:?}");

    let sender_balance_after = era_provider
        .get_balance(zk_wallet.l2_address(), None)
        .await?;
    let receiver_balance_after = era_provider.get_balance(args.to, None).await?;

    log::info!("Sender balance after: {sender_balance_after}");
    log::info!("Receiver balance after: {receiver_balance_after}");
    Ok(())
}
