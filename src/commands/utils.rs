use crate::config::ZKSyncConfig;
use clap::Subcommand;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    abi::{parse_abi_str, ParamType, Token, Tokenizable},
    types::{Address, Bytes, I256, U128, U256},
};

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[command(name = "calldata", visible_alias = "cd")]
    CalldataEncode {
        signature: String,
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(visible_alias = "cdd")]
    CalldataDecode {
        signature: String,
        #[arg(allow_hyphen_values = true)]
        calldata: Bytes,
    },
}

impl Command {
    pub fn run(self, _cfg: ZKSyncConfig) -> eyre::Result<()> {
        match self {
            Command::CalldataEncode { signature, args } => {
                let abi = parse_abi_str(signature.as_str())?;
                let function = abi.functions().next().context("No functions found")?;
                let args = function
                    .inputs
                    .clone()
                    .into_iter()
                    .zip(args.iter())
                    .map(|(param, raw_param)| parse_param_into_token(param.kind, raw_param))
                    .collect::<eyre::Result<Vec<Token>>>()?;
                let encoded = function.encode_input(&args)?;
                println!("0x{}", hex::encode(encoded));
            }
            Command::CalldataDecode {
                signature,
                calldata,
            } => {
                let abi = parse_abi_str(signature.as_str())?;
                let function = abi.functions().next().context("No functions found")?;
                let decoded = function.decode_input(
                    calldata
                        .get(4..)
                        .context("Could not remove function selector from calldata")?,
                )?;
                for token in decoded {
                    display_token(token)?;
                }
            }
        };
        Ok(())
    }
}

fn parse_param_into_token(param_kind: ParamType, raw_param: &str) -> eyre::Result<Token> {
    match param_kind {
        zksync_ethers_rs::abi::ParamType::Address => raw_param
            .parse::<Address>()
            .map(Tokenizable::into_token)
            .map_err(Into::<eyre::Error>::into),
        zksync_ethers_rs::abi::ParamType::Bytes => raw_param
            .parse::<Bytes>()
            .map(Tokenizable::into_token)
            .map_err(Into::<eyre::Error>::into),
        zksync_ethers_rs::abi::ParamType::Int(size) => match size {
            8 => raw_param
                .parse::<i8>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            16 => raw_param
                .parse::<i16>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            32 => raw_param
                .parse::<i32>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            64 => raw_param
                .parse::<i64>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            128 => raw_param
                .parse::<i128>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            256 => raw_param
                .parse::<I256>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            _ => unreachable!(),
        },
        zksync_ethers_rs::abi::ParamType::Uint(size) => match size {
            8 => raw_param
                .parse::<u8>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            16 => raw_param
                .parse::<u16>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            32 => raw_param
                .parse::<u32>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            64 => raw_param
                .parse::<u64>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            128 => raw_param
                .parse::<U128>()
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            256 => U256::from_dec_str(raw_param)
                .map(Tokenizable::into_token)
                .map_err(Into::<eyre::Error>::into),
            _ => unreachable!(),
        },
        zksync_ethers_rs::abi::ParamType::Bool => raw_param
            .parse::<bool>()
            .map(Tokenizable::into_token)
            .map_err(Into::<eyre::Error>::into),
        zksync_ethers_rs::abi::ParamType::String => Ok(Token::String(raw_param.to_owned())),
        zksync_ethers_rs::abi::ParamType::Array(_param_type) => todo!(),
        zksync_ethers_rs::abi::ParamType::FixedBytes(_size) => todo!(),
        zksync_ethers_rs::abi::ParamType::FixedArray(_param_type, _size) => {
            todo!()
        }
        zksync_ethers_rs::abi::ParamType::Tuple(param_types) => {
            let parsed_params = param_types
                .into_iter()
                .map(|param_type| parse_param_into_token(param_type, raw_param))
                .collect::<eyre::Result<Vec<Token>>>()?;
            Ok(Token::Tuple(parsed_params))
        }
    }
}

fn display_token(token: Token) -> eyre::Result<()> {
    match token {
        Token::Address(_) => println!("{:?}", token.into_address().context("Address")?),
        Token::FixedBytes(_) => println!("{:?}", token.into_fixed_bytes().context("FixedBytes")?),
        Token::Bytes(_) => println!("{:?}", token.into_bytes().context("Bytes")?),
        Token::Int(_) => println!("{:?}", token.into_int().context("Int")?),
        Token::Uint(_) => println!("{:?}", token.into_uint().context("Uint")?),
        Token::Bool(_) => println!("{:?}", token.into_bool().context("Bool")?),
        Token::String(_) => println!("{:?}", token.into_string().context("String")?),
        Token::FixedArray(_) => println!("{:?}", token.into_fixed_array().context("FixedArray")?),
        Token::Array(_) => println!("{:?}", token.into_array().context("Array")?),
        Token::Tuple(_) => println!("{:?}", token.into_tuple().context("Tuple")?),
    };
    Ok(())
}
