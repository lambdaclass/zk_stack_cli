use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use std::sync::Arc;
use zksync_ethers_rs::{
    contracts::{erc20::ERC20, l2_shared_bridge::get_l2_token_from_l1_address},
    core::utils::{format_ether, format_units},
    providers::{Middleware, Provider},
    types::Address,
    utils::L2_ETH_TOKEN_ADDRESS,
    ZKMiddleware,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "token")]
    pub token_address: Option<Address>,
    #[clap(long = "tol2", required = false)]
    pub l2_token: bool,
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

    let provider = if args.l1 {
        l1_provider.clone()
    } else {
        l2_provider.clone()
    };

    let wallet_address = cfg.wallet.context("Wallet config missing")?.address;
    let network = if args.l1 { "L1" } else { "L2" };

    if let Some(mut token_address) = args.token_address {
        if args.l2_token && !args.l1 {
            token_address = get_l2_token_from_l1_address(token_address, &l2_provider).await;
        }
        let erc20 = ERC20::new(token_address, Arc::new(provider));
        let balance = erc20.balance_of(wallet_address).await?;
        let token_symbol = erc20.symbol().await?;
        let token_decimals: i32 = erc20.decimals().await?.into();
        let parsed_balance = format_units(balance, token_decimals)?;
        println!("[{network}] Balance: {parsed_balance} {token_symbol}");
    } else {
        match network {
            "L1" => {
                let balance = provider.get_balance(wallet_address, None).await?;
                let parsed_balance = format_ether(balance);
                println!("[{network}] Balance: {parsed_balance} ETH");
            }
            "L2" => {
                let base_token_address = l2_provider.get_base_token_l1_address().await?;
                println!("Base Token Address: {base_token_address}");
                let balance = provider.get_balance(wallet_address, None).await?;
                if base_token_address != L2_ETH_TOKEN_ADDRESS {
                    let erc20 = ERC20::new(base_token_address, Arc::new(l1_provider));
                    let token_symbol = erc20.symbol().await?;
                    let token_decimals: i32 = erc20.decimals().await?.into();
                    let parsed_balance = format_units(balance, token_decimals)?;
                    println!("[{network}] Base Token Balance: {parsed_balance} {token_symbol}");
                } else {
                    let parsed_balance = format_ether(balance);
                    println!("[{network}] Base Token Balance: {parsed_balance} ETH");
                }
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}
