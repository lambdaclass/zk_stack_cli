use clap::Args as ClapArgs;
use zksync_web3_rs::abi::HumanReadableParser;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "FUNCTION_SIGNATURE")]
    pub function_signature: String,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let function = HumanReadableParser::parse_function(&args.function_signature)?;
    let short_signature = hex::encode(function.short_signature());
    let mut selector = String::from("0x");
    selector.push_str(&short_signature);
    log::info!("{selector:?}");
    Ok(())
}
