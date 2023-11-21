use clap::Args as ClapArgs;
use eyre::eyre;
use std::str::FromStr;
use zksync_web3_rs::{
    prelude::abi::{encode, HumanReadableParser, Token},
    types::{Address, U256},
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
        "uint256" => Ok(Token::Uint(U256::from_str(arg)?)),
        "sint256" => Ok(Token::Int(U256::from_str(arg)?)),
        "address" => Ok(Token::Address(Address::from_str(arg)?)),
        "bool" => Ok(Token::Bool(arg.parse::<bool>()?)),
        "bytes" => Ok(Token::Bytes(hex::decode(arg)?)),
        "string" => Ok(Token::String(arg.to_owned())),
        // "[]uint256" => Ok(Token::Array(Token::Uint(U256::from_str(arg)?))),
        // "[]sint256" => Ok(Token::Array(Token::Int(U256::from_str(arg)?))),
        // "[]address" => Ok(Token::Array(Token::Address)),
        // "[]bool" => Ok(Token::Array(Token::Bool)),
        // "[]bytes" => Ok(Token::Array(Token::Bytes)),
        // "[]string" => Ok(Token::Array(Token::String)),
        other => Err(eyre!("Unable to parse output type: {other}")),
    }?;

    Ok(x)
}
