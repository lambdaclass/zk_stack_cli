// TODO
// THIS FUNCTIONS SHOULD BE MIGRATED TO THE zksync_ethers_rs
use eyre::ContextCompat;
use zksync_ethers_rs::{
    abi::Token,
    core::utils::keccak256,
    types::{Address, U256},
};

pub(crate) fn encode_call(
    selector: Option<[u8; 4]>,
    bytecode: Option<Vec<u8>>,
    args: Option<Vec<String>>,
    types: Option<Vec<String>>,
) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();

    if let Some(s) = selector {
        data.extend_from_slice(&s);
    } else if let Some(b) = bytecode {
        data.extend_from_slice(&b);
    }

    let tokens = match (args, types) {
        (Some(a), Some(t)) => {
            if a.len() != t.len() {
                return Err(eyre::eyre!("Argument count does not match type count"));
            }
            a.into_iter()
                .zip(t.into_iter())
                .map(|(arg, arg_type)| {
                    match arg_type.as_str() {
                        "address" => {
                            // Parse as Address
                            let address: Address = arg
                                .parse()
                                .map_err(|_e| eyre::eyre!("Invalid address format"))?;
                            Ok(Token::Address(address))
                        }
                        "uint256" => {
                            // Parse as uint256
                            let value: U256 = U256::from_dec_str(&arg)
                                .map_err(|_e| eyre::eyre!("Invalid uint256 format"))?;
                            Ok(Token::Uint(value))
                        }
                        "string" => {
                            // Parse as string
                            Ok(Token::String(arg))
                        }
                        _ => Err(eyre::eyre!("Unsupported argument type")),
                    }
                })
                .collect::<Result<Vec<Token>, eyre::Report>>()?
        }
        (Some(_), None) => {
            return Err(eyre::eyre!("Types not provided"));
        }
        (None, _) => vec![],
    };

    let encoded_args = zksync_ethers_rs::abi::encode(&tokens);
    data.extend(encoded_args);
    Ok(data)
}

pub(crate) fn parse_signature(signature: &str) -> eyre::Result<Option<Vec<String>>> {
    let mut types = Vec::new();

    if let Some(start) = signature.find('(') {
        if let Some(end) = signature.rfind(')') {
            let params = signature.get(start + 1..end).context("Parsing Error")?;

            // Split the parameters by comma
            for param in params.split(',') {
                let trimmed_param = param.trim();
                types.push(trimmed_param.to_owned())
            }
        } else {
            return Err(eyre::eyre!("Missing closing parenthesis in signature"));
        }
    } else {
        return Err(eyre::eyre!("Missing opening parenthesis in signature"));
    }
    if types.is_empty() {
        return Ok(None);
    }
    Ok(Some(types))
}

pub(crate) fn get_fn_selector(function_signature: &str) -> [u8; 4] {
    let hash = keccak256(function_signature.as_bytes());
    let mut selector = [0_u8; 4];
    selector.copy_from_slice(&hash[0..4]);
    selector
}
