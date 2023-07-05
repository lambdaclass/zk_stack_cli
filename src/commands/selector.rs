use clap::Args;
use zksync_web3_rs::abi::HumanReadableParser;

#[derive(Args)]
pub(crate) struct SelectorArgs {
    #[clap(long, name = "FUNCTION_SIGNATURE")]
    pub function_signature: String,
}

pub(crate) async fn run(args: SelectorArgs) -> eyre::Result<()> {
    let function = HumanReadableParser::parse_function(&args.function_signature)?;
    let short_signature = hex::encode(function.short_signature());
    let mut selector = String::from("0x");
    selector.push_str(&short_signature);
    log::info!("{selector:?}");
    Ok(())
}
