// use crate::config::ZKSyncConfig;
// use clap::Args as ClapArgs;
// use eyre::ContextCompat;
// use zksync_ethers_rs::{
//     providers::{Middleware, Provider},
//     signers::{LocalWallet, Signer},
//     types::{Address, U256},
//     utils::parse_units,
//     zks_wallet::WithdrawRequest,
//     ZKMiddleware, ZKSWallet,
// };

// #[derive(ClapArgs)]
// pub struct Args {
//     #[clap(short, long, name = "AMOUNT_TO_WITHDRAW_IN_ETHER")]
//     amount: U256,
//     #[clap(short, long, name = "SENDER_PRIVATE_KEY")]
//     from: LocalWallet,
//     #[clap(short, long, name = "RECEIVER_ADDRESS")]
//     to: Option<Address>,
//     #[clap(short, long, name = "CHAIN_ID")]
//     chain_id: u16,
// }

// pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
//     let l1_provider = if let Some(port) = config.l1_port {
//         Provider::try_from(format!(
//             "http://{host}:{port}",
//             host = config.host,
//             port = port
//         ))?
//     } else {
//         Provider::try_from(config.host.to_owned())?
//     }
//     .interval(std::time::Duration::from_millis(10));
//     let l2_provider = if let Some(port) = config.l2_port {
//         Provider::try_from(format!(
//             "http://{host}:{port}",
//             host = config.host,
//             port = port
//         ))?
//     } else {
//         Provider::try_from(config.host.to_owned())?
//     }
//     .interval(std::time::Duration::from_millis(10));

//     let wallet = args.from.with_chain_id(args.chain_id);
//     let zk_wallet = ZKSWallet::new(wallet, None, Some(l2_provider.clone()), Some(l1_provider))?;

//     // See balances before withdraw
//     let l1_balance_before = zk_wallet.eth_balance().await?;
//     let l2_balance_before = if let Some(to) = args.to {
//         l2_provider.get_balance(to, None).await?
//     } else {
//         zk_wallet.era_balance().await?
//     };

//     log::info!("Balance on L1 before withdrawal: {l1_balance_before}");
//     log::info!("Balance on L2 before withdrawal: {l2_balance_before}");

//     // Withdraw
//     let amount_to_withdraw: U256 = parse_units(args.amount, "ether")?.into();

//     let withdraw_request = WithdrawRequest::new(amount_to_withdraw).to(zk_wallet.l1_address());
//     let tx_hash = zk_wallet.withdraw(&withdraw_request).await?;
//     let tx_receipt = zk_wallet
//         .get_era_provider()?
//         .wait_for_finalize(tx_hash, None, None)
//         .await?;

//     log::info!("L2 Transaction hash: {:?}", tx_receipt.transaction_hash);

//     let tx_finalize_hash = zk_wallet.finalize_withdraw(tx_hash).await?;

//     let tx_finalize_receipt = zk_wallet
//         .get_eth_provider()?
//         .get_transaction_receipt(tx_finalize_hash)
//         .await?
//         .context("Failed to get transaction receipt")?;
//     log::info!(
//         "L1 Transaction hash: {:?}",
//         tx_finalize_receipt.transaction_hash
//     );

//     // See balances after withdraw
//     let l1_balance_after_finalize = zk_wallet.eth_balance().await?;
//     let l2_balance_after_finalize = if let Some(to) = args.to {
//         l2_provider.get_balance(to, None).await?
//     } else {
//         zk_wallet.era_balance().await?
//     };

//     log::info!("Balance on L1 after finalize withdraw: {l1_balance_after_finalize}");
//     log::info!("Balance on L2 after finalize withdraw: {l2_balance_after_finalize}");

//     Ok(())
// }
