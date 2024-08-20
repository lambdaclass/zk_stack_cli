use crate::{commands::contracts::governance::parse_operation, config::ZKSyncConfig};
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    abi::Hash,
    contracts::governance::{Governance, Operation},
    providers::Middleware,
    types::U256,
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(short = 's', long, conflicts_with_all = &["transparent", "operation"], requires = "operation_id", required_unless_present = "transparent")]
    pub shadow: bool,
    #[clap(long, conflicts_with_all = &["transparent", "operation"], requires = "shadow", required_unless_present = "transparent",)]
    pub operation_id: Option<Hash>,
    #[clap(long, default_value = "0", value_parser = U256::from_dec_str, required = false)]
    pub delay: U256,
    #[clap(short = 't', long, conflicts_with_all = &["shadow", "operation_id"], requires = "operation", required_unless_present = "shadow")]
    pub transparent: bool,
    #[clap(long, conflicts_with_all = &["shadow", "operation_id"], value_parser = parse_operation, requires = "transparent", required = false)]
    pub operation: Option<Operation>,
    #[clap(long, required = false)]
    pub explorer_url: bool,
}

pub(crate) async fn run(
    args: Args,
    governance: Governance<impl Middleware + 'static>,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    let transaction_receipt = if args.shadow {
        governance
            .schedule_shadow(
                args.operation_id
                    .context("--operation-id is required with --shadow")?
                    .into(),
                args.delay,
            )
            .send()
            .await?
            .await?
            .context("No transaction receipt for shadow operation")?
    } else if args.transparent {
        governance
            .schedule_transparent(
                args.operation
                    .context("--operation is required with --transparent")?,
                args.delay,
            )
            .send()
            .await?
            .await?
            .context("No transaction receipt for transparent operation")?
    } else {
        eyre::bail!("Either --shadow or --transparent must be provided");
    };
    if args.explorer_url {
        let url = cfg
            .network
            .l1_explorer_url
            .context("L1 Explorer URL missing in config")?;
        println!(
            "Upgrade scheduled: {url}/tx/{:?}",
            transaction_receipt.transaction_hash
        );
    } else {
        println!(
            "Upgrade scheduled: {:?}",
            transaction_receipt.transaction_hash
        );
    }
    Ok(())
}
