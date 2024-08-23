use crate::utils::balance::display_balance;
use crate::utils::wallet::new_zkwallet;
use colored::Colorize;
use eyre::ContextCompat;
use std::ops::Div;
use std::sync::Arc;
use tokio::task::JoinSet;
use zksync_ethers_rs::contracts::erc20::MINT_IERC20;
use zksync_ethers_rs::core::rand::thread_rng;
use zksync_ethers_rs::signers::{LocalWallet, Signer};
use zksync_ethers_rs::types::H256;
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

pub async fn send_transactions(
    from_wallet: &Arc<ZKWallet<Provider<Http>, LocalWallet>>,
    to_wallets: &Vec<Arc<ZKWallet<Provider<Http>, LocalWallet>>>,
    parsed_amount: U256,
) -> eyre::Result<Vec<H256>> {
    let mut l2_txs_receipts: Vec<H256> = Vec::new();
    let mut set = JoinSet::new();

    let mut nonce = from_wallet
        .l2_provider()
        .get_transaction_count(from_wallet.l2_address(), None)
        .await?;

    for w in to_wallets {
        let from_wallet_clone = Arc::clone(from_wallet);
        let to = w.l2_address();
        set.spawn(async move {
            from_wallet_clone
                .transfer_base_token(parsed_amount, to, Some(L2TxOverrides::new().nonce(nonce)))
                .await
        });
        nonce = nonce.saturating_add(U256::one());
    }

    while let Some(res) = set.join_next().await {
        let tx_hash = res??;
        l2_txs_receipts.push(tx_hash);
    }
    Ok(l2_txs_receipts)
}

pub async fn send_transactions_back(
    from_wallets: &Vec<Arc<ZKWallet<Provider<Http>, LocalWallet>>>,
    to_wallet: &Arc<ZKWallet<Provider<Http>, LocalWallet>>,
) -> eyre::Result<Vec<H256>> {
    let mut l2_txs_receipts: Vec<H256> = Vec::new();
    let mut set = JoinSet::new();

    for w in from_wallets {
        let to_wallet_clone = Arc::clone(to_wallet);
        let from_wallet_clone = Arc::clone(w);
        set.spawn(async move {
            future_transfer_base_token_back(&from_wallet_clone, &to_wallet_clone).await
        });
    }

    while let Some(res) = set.join_next().await {
        let tx_hash = res??;
        l2_txs_receipts.push(tx_hash);
    }
    Ok(l2_txs_receipts)
}

pub async fn future_transfer_base_token_back(
    from_wallet: &ZKWallet<Provider<Http>, LocalWallet>,
    to_wallet: &ZKWallet<Provider<Http>, LocalWallet>,
) -> eyre::Result<H256> {
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
    Ok(transfer_hash)
}

pub async fn deposit_base_token(
    from_wallet: &Arc<ZKWallet<Provider<Http>, LocalWallet>>,
    parsed_amount_to_deposit: U256,
) -> eyre::Result<()> {
    println!(
        "{} Deposit from {} wallet to {} wallet.",
        "[L1->L2]".bold().bright_cyan().on_black(),
        "rich".bold().red().on_black(),
        "rich".bold().red().on_black()
    );
    from_wallet
        .deposit_base_token(parsed_amount_to_deposit)
        .await?;
    Ok(())
}

pub async fn get_n_random_wallets(
    number_of_wallets: u16,
    l1_provider: &Provider<Http>,
    l2_provider: &Provider<Http>,
) -> eyre::Result<Vec<Arc<ZKWallet<Provider<Http>, LocalWallet>>>> {
    let mut wallets = Vec::new();
    for i in 1..=number_of_wallets {
        let local_wallet = LocalWallet::new(&mut thread_rng());
        let pk_bytes = local_wallet.signer().to_bytes();
        let pk = hex::encode(pk_bytes);
        println!(
            "Wallet [{i:0>3}] addr: {:?} || pk: 0x{pk}",
            local_wallet.address(),
        );
        let w = new_zkwallet(local_wallet, l1_provider, l2_provider).await?;
        wallets.push(Arc::new(w));
    }
    Ok(wallets)
}

pub async fn display_balances(
    wallets: &[Arc<ZKWallet<Provider<Http>, LocalWallet>>],
) -> eyre::Result<()> {
    for (i, w) in wallets.iter().enumerate() {
        println!("{}", "=".repeat(64));
        println!("Wallet [{i:0>3}] addr: {:?}", w.l2_address());
        display_balance(None, w, false, true).await?;
        println!("{}", "=".repeat(64));
    }
    Ok(())
}

pub async fn erc20_l1_mint(
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
