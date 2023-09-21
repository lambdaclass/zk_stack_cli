use clap::Args as ClapArgs;
use eyre::eyre;
use std::str::FromStr;
use zksync_web3_rs::{
    prelude::abi::{encode, HumanReadableParser, Token, Tokenizable},
    types::U256,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "FUNCTION_SIGNATURE")]
    pub function: Option<String>,
    #[clap(long, num_args(1..), name = "VALUE")]
    pub arguments: Vec<String>,
    #[clap(long, num_args(1..), name = "VALUE_TYPE")]
    pub types: Vec<String>,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let parsed_arguments = args
        .arguments
        .iter()
        .zip(args.types.iter())
        .map(|(arg, t)| parse_token(arg, t))
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

fn parse_token(arg: &str, t: &str) -> eyre::Result<Token> {
    let x = match t {
        "uint256" => Ok(U256::from_str(arg).map(Tokenizable::into_token)),
        other => Err(eyre!("Could not parse type: {other}")),
    }??;

    Ok(x)
}
