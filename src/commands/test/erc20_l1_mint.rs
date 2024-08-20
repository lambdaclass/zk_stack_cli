#![allow(clippy::indexing_slicing)]
use std::sync::Arc;

use crate::{
    commands::utils::{
        balance::{display_l1_balance, get_erc20_balance_decimals_symbol},
        wallet::new_zkwallet,
    },
    config::ZKSyncConfig,
};
use clap::Args as ClapArgs;
use ethers::{providers::Http, types::TransactionReceipt};
use eyre::ContextCompat;
use spinoff::{spinners, Color, Spinner};
use zksync_ethers_rs::{
    contract::abigen,
    core::{k256::ecdsa::SigningKey, utils::parse_ether},
    middleware::SignerMiddleware,
    providers::{Middleware, Provider},
    signers::{Signer, Wallet},
    types::{Address, U256},
    zk_wallet::ZKWallet,
    ZKMiddleware,
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "token")]
    pub token_address: Option<Address>,
    #[clap(long = "amount", short = 'a', default_value = "10")]
    pub amount: String,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let base_token_address = l2_provider.get_base_token_l1_address().await?;

    let token_address = args.token_address.unwrap_or(base_token_address);

    let wallet_config = cfg.wallet.context("Wallet config missing")?;
    let wallet = wallet_config.private_key.parse::<Wallet<SigningKey>>()?;

    let zk_wallet = new_zkwallet(wallet, &l1_provider, &l2_provider).await?;

    let (_, _, token_symbol) =
        get_erc20_balance_decimals_symbol(token_address, zk_wallet.l1_address(), &l1_provider)
            .await?;

    let address = zk_wallet.l1_address();
    let future_receipt = erc20_l1_mint(token_address, zk_wallet, parse_ether(&args.amount)?);
    display_l1_balance(Some(token_address), &l1_provider, address).await?;
    let msg = format!("Minting {} {token_symbol}", args.amount);
    let mut spinner = Spinner::new(spinners::Dots, msg, Color::Blue);
    let receipt = future_receipt.await?;
    spinner.success("Tokens Minted!");
    println!("Transaction Hash: {:?}", receipt.transaction_hash);
    display_l1_balance(Some(token_address), &l1_provider, address).await?;
    Ok(())
}

abigen!(
    MINT_IERC20,
    "[function mint(address _to, uint256 _amount) public returns (bool)]"
);

pub(crate) async fn erc20_l1_mint(
    erc20_token_address: Address,
    wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    let erc20_contract = MINT_IERC20::new(erc20_token_address, wallet.l1_signer());
    let tx_receipt = erc20_contract
        .mint(wallet.l1_address(), amount)
        .send()
        .await?
        .await?
        .context("No transaction receipt for erc20 mint")?;

    Ok(tx_receipt)
}
