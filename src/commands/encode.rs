use clap::Args;
use std::str::FromStr;
use zksync_web3_rs::{
    abi::{encode, HumanReadableParser, Token, Tokenizable},
    types::U256,
};

#[derive(Args)]
pub(crate) struct EncodeArgs {
    #[clap(long, name = "FUNCTION_SIGNATURE")]
    pub function: Option<String>,
    #[clap(long, num_args(1..), name = "VALUE")]
    pub arguments: Vec<String>,
    #[clap(long, num_args(1..), name = "VALUE_TYPE")]
    pub types: Vec<String>,
}

pub(crate) async fn run(args: EncodeArgs) -> eyre::Result<()> {
    let parsed_arguments = args
        .arguments
        .iter()
        .zip(args.types.iter())
        .map(|(arg, t)| match t.as_str() {
            "uint256" => U256::from_str(&arg).map(Tokenizable::into_token),
            _ => todo!(),
        })
        .collect::<Result<Vec<Token>, _>>()?;

    let encoded = if let Some(function_signature) = args.function {
        let function = HumanReadableParser::parse_function(&function_signature)?;
        hex::encode(function.encode_input(&parsed_arguments)?)
    } else {
        hex::encode(encode(&parsed_arguments))
    };

    let mut binding = String::from("0x");
    binding.push_str(&encoded);

    log::info!("{binding:?}");

    Ok(())
}
