use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use std::sync::Arc;
use zksync_ethers_rs::{
    contracts::{erc20::ERC20, l2_shared_bridge::get_l2_token_from_l1_address},
    core::utils::{format_ether, format_units},
    providers::{Http, Middleware, Provider},
    types::Address,
    utils::L2_ETH_TOKEN_ADDRESS,
    ZKMiddleware,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "token")]
    pub token_address: Option<Address>,
    #[clap(long = "l2", required = false)]
    pub l2: bool,
    #[clap(long = "l1", required = false)]
    pub l1: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;

    let wallet_address = cfg.wallet.context("Wallet config missing")?.address;

    let network = match (args.l1, args.l2) {
        (false, false) | (false, true) => "L2",
        (true, false) => "L1",
        (true, true) => "BOTH",
    };

    let base_token_address = l2_provider.get_base_token_l1_address().await?;

    match network {
        "L1" => {
            if let Some(token_address) = args.token_address {
                let (parsed_balance, _, token_symbol) = get_erc20_balance_decimals_symbol(
                    token_address,
                    wallet_address,
                    l1_provider.clone(),
                )
                .await?;
                println!("[{network}] Balance: {parsed_balance} {token_symbol}");
            } else {
                let balance = l1_provider.get_balance(wallet_address, None).await?;
                let parsed_balance = format_ether(balance);
                println!("[{network}] Balance: {parsed_balance} ETH");
            }
        }
        "L2" => {
            println!("Base Token Address: {base_token_address:?}");
            if let Some(token_address) = args.token_address {
                if token_address == base_token_address {
                    print_l2_base_token_balance(
                        base_token_address,
                        wallet_address,
                        l2_provider.clone(),
                        l1_provider.clone(),
                    )
                    .await?;
                } else {
                    let (parsed_balance, _, token_symbol) = get_erc20_balance_decimals_symbol(
                        token_address,
                        wallet_address,
                        l2_provider.clone(),
                    )
                    .await?;
                    println!("[{network}] Balance: {parsed_balance} {token_symbol}");
                }
            } else {
                print_l2_base_token_balance(
                    base_token_address,
                    wallet_address,
                    l2_provider.clone(),
                    l1_provider.clone(),
                )
                .await?;
            }
        }
        "BOTH" => {
            println!("Base Token Address: {base_token_address:?}");
            if let Some(token_address) = args.token_address {
                // L2
                let l2_token_address =
                    get_l2_token_from_l1_address(token_address, &l2_provider).await;
                if token_address == base_token_address {
                    print_l2_base_token_balance(
                        base_token_address,
                        wallet_address,
                        l2_provider.clone(),
                        l1_provider.clone(),
                    )
                    .await?;
                } else {
                    let (parsed_balance, _, token_symbol) = get_erc20_balance_decimals_symbol(
                        l2_token_address,
                        wallet_address,
                        l2_provider.clone(),
                    )
                    .await?;
                    println!("[L2] Balance: {parsed_balance} {token_symbol}");
                }
                // L1
                let (parsed_balance, _, token_symbol) = get_erc20_balance_decimals_symbol(
                    token_address,
                    wallet_address,
                    l1_provider.clone(),
                )
                .await?;
                println!("[L1] Balance: {parsed_balance} {token_symbol}");
            } else {
                // L2
                print_l2_base_token_balance(
                    base_token_address,
                    wallet_address,
                    l2_provider.clone(),
                    l1_provider.clone(),
                )
                .await?;
                // L1
                let balance = l1_provider.get_balance(wallet_address, None).await?;
                let parsed_balance = format_ether(balance);
                println!("[L1] Balance: {parsed_balance} ETH");
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

async fn get_erc20_balance_decimals_symbol(
    token_address: Address,
    wallet_address: Address,
    provider: Provider<Http>,
) -> eyre::Result<(String, i32, String)> {
    let erc20 = ERC20::new(token_address, Arc::new(provider));
    let balance = erc20.balance_of(wallet_address).await?;
    let token_symbol = erc20.symbol().await?;
    let token_decimals: i32 = erc20.decimals().await?.into();
    let parsed_balance = format_units(balance, token_decimals)?;
    Ok((parsed_balance, token_decimals, token_symbol))
}

async fn print_l2_base_token_balance(
    base_token_address: Address,
    wallet_address: Address,
    l2_provider: Provider<Http>,
    l1_provider: Provider<Http>,
) -> eyre::Result<()> {
    let balance = l2_provider.get_balance(wallet_address, None).await?;
    if base_token_address != L2_ETH_TOKEN_ADDRESS {
        let (_, token_decimals, token_symbol) = get_erc20_balance_decimals_symbol(
            base_token_address,
            wallet_address,
            l1_provider.clone(),
        )
        .await?;
        let parsed_balance = format_units(balance, token_decimals)?;
        println!("[L2] Base Token Balance: {parsed_balance} {token_symbol}");
    } else {
        let parsed_balance = format_ether(balance);
        println!("[L2] Base Token Balance: {parsed_balance} ETH");
    }
    Ok(())
}
