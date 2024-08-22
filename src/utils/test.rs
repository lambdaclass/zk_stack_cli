use crate::utils::balance::display_balance;
use crate::utils::wallet::new_zkwallet;
use eyre::ContextCompat;
use std::ops::Div;
use zksync_ethers_rs::contracts::erc20::MINT_IERC20;
use zksync_ethers_rs::core::rand::thread_rng;
use zksync_ethers_rs::signers::{LocalWallet, Signer};
use zksync_ethers_rs::{
    core::k256::ecdsa::SigningKey,
    providers::{Http, Middleware, Provider},
    signers::Wallet,
    types::{
        transaction::eip2718::TypedTransaction, Address, Eip1559TransactionRequest, L2TxOverrides,
        TransactionReceipt, U256,
    },
    zk_wallet::ZKWallet,
};

pub async fn future_transfer_base_token(
    from_wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    to_wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    parsed_amount: U256,
    overrides: Option<L2TxOverrides>,
) -> eyre::Result<()> {
    display_balance(None, to_wallet, false, true).await?;

    let transfer_hash = from_wallet
        .transfer_base_token(parsed_amount, to_wallet.l2_address(), overrides)
        .await?;

    println!("Transfer hash: {transfer_hash:?}");

    display_balance(None, to_wallet, false, true).await?;

    Ok(())
}

pub async fn future_transfer_base_token_back(
    from_wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    to_wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
) -> eyre::Result<()> {
    display_balance(None, from_wallet, false, true).await?;
    display_balance(None, to_wallet, false, true).await?;
    let balance = from_wallet
        .l2_provider()
        .get_balance(from_wallet.l2_address(), None)
        .await?;
    let transfer_tx = TypedTransaction::Eip1559(
        Eip1559TransactionRequest::new()
            .from(from_wallet.l2_address())
            .to(to_wallet.l2_address())
            .value(balance),
    );
    let gas_estimate = from_wallet
        .l2_provider()
        .estimate_gas(&transfer_tx, None)
        .await?
        .div(10_u32)
        .saturating_mul(U256::from(11_u32)); // 10% of headroom
    let gas_price = from_wallet.l2_provider().get_gas_price().await?;
    let gas = gas_estimate.saturating_mul(gas_price);
    let transfer_hash = from_wallet
        .transfer_base_token(
            balance.saturating_sub(gas),
            to_wallet.l1_address(),
            // The nonce is not changed since all the transfers are from different wallets
            None,
        )
        .await?;
    println!("Transfer hash: {transfer_hash:?}");
    display_balance(None, from_wallet, false, true).await?;
    display_balance(None, to_wallet, false, true).await?;
    Ok(())
}

pub(crate) async fn get_n_random_wallets(
    number_of_wallets: u16,
    l1_provider: &Provider<Http>,
    l2_provider: &Provider<Http>,
) -> eyre::Result<Vec<ZKWallet<Provider<Http>, LocalWallet>>> {
    let mut wallets = Vec::new();
    for i in 1..=number_of_wallets {
        let local_wallet = LocalWallet::new(&mut thread_rng());
        let pk_bytes = local_wallet.signer().to_bytes();
        //let sk: SigningKey = SigningKey::from_bytes(&pk_bytes)?;
        let pk = hex::encode(pk_bytes);
        println!(
            "Wallet [{i:0>3}] addr: {:?} || pk: 0x{pk}",
            local_wallet.address(),
        );
        let w = new_zkwallet(local_wallet, l1_provider, l2_provider).await?;
        wallets.push(w);
    }
    Ok(wallets)
}

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
