use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_web3_rs::providers::Provider;
use zksync_web3_rs::types::{Address, H256, U64};
use zksync_web3_rs::zks_provider::ZKSProvider;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "TRANSACTION_HASH")]
    transaction: H256,
    #[clap(long, name = "L2_TO_L1_LOG_INDEX")]
    log_index: Option<u64>,
    #[clap(
        long,
        action,
        conflicts_with = "msg_proof, block, sender, msg",
        group = "log",
        name = "LOG_PROOF"
    )]
    log_proof: bool,
    #[clap(
        long,
        action,
        conflicts_with = "log_proof, l2_to_l1_log_index",
        group = "msg",
        name = "MESSAGE_PROOF"
    )]
    msg_proof: bool,
    #[clap(
        long,
        conflicts_with = "log_proof, l2_to_l1_log_index",
        group = "msg",
        name = "MESSAGE_BLOCK"
    )]
    block: U64,
    #[clap(
        long,
        conflicts_with = "log_proof, l2_to_l1_log_index",
        group = "msg",
        name = "MESSAGE_SENDER"
    )]
    sender: Address,
    #[clap(
        long,
        conflicts_with = "log_proof, l2_to_l1_log_index",
        group = "msg",
        name = "MESSAGE"
    )]
    msg: H256,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = if let Some(port) = config.l2_port {
        Provider::try_from(format!("http://{host}:{port}", host = config.host))?
    } else {
        Provider::try_from(config.host.clone())?
    }
    .interval(std::time::Duration::from_millis(10));
    let proof = if args.log_proof {
        provider
            .get_l2_to_l1_log_proof(args.transaction, args.log_index)
            .await?
    } else if args.msg_proof {
        provider
            .get_l2_to_l1_msg_proof(args.block, args.sender, args.msg, args.log_index)
            .await?
    } else {
        eyre::bail!("no type of proof provided")
    }
    .context("no proof");
    log::info!("{proof:#?}");
    Ok(())
}
