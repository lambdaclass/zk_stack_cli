use crate::config::ZKSyncConfig;
use crate::utils::balance::{display_balance, get_erc20_balance_decimals_symbol};
use crate::utils::wallet::*;
use clap::Subcommand;
use colored::*;
use eyre::ContextCompat;
use spinoff::{spinners, Color, Spinner};
use std::ops::Div;
use zksync_ethers_rs::contracts::erc20::MINT_IERC20;
use zksync_ethers_rs::core::utils::format_ether;
use zksync_ethers_rs::{
    core::{k256::ecdsa::SigningKey, rand::thread_rng, utils::parse_ether},
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{
        transaction::eip2718::TypedTransaction, Address, Eip1559TransactionRequest, L2TxOverrides,
        TransactionReceipt, U256,
    },
    wait_for_finalize_withdrawal,
    zk_wallet::ZKWallet,
    ZKMiddleware,
};

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "LoadTest the zkStack Chain.")]
    LoadTest {
        #[clap(long = "wallets", short = 'w', required = true)]
        number_of_wallets: u16,
        #[clap(
            long = "amount",
            short = 'a',
            required = true,
            help = "Amount of BaseToken to deposit, 20% more will be deposited.\nThat extra 20% will remain in the main wallet,\nthe rest will be redistributed to random wallets"
        )]
        amount: f32,
        #[clap(
            long = "reruns",
            short = 'r',
            help = "If set to 0 it will run indefinitely, If not set defaults to 1 run."
        )]
        reruns_wanted: Option<u8>,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let wallet_config = cfg.wallet.clone().context("Wallet config missing")?;
        match self {
            Command::LoadTest {
                number_of_wallets,
                amount,
                reruns_wanted,
            } => {
                let (zk_wallet, l1_provider, l2_provider) = get_wallet_l1_l2_providers(cfg)?;
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
                    let w = new_zkwallet(local_wallet, &l1_provider, &l2_provider).await?;
                    wallets.push(w);
                }

                let base_token_address = l2_provider.get_base_token_l1_address().await?;

                // ideally it should be the amount transferred, the gas + fees have to be deducted automatically
                let parsed_amount_to_deposit = parse_ether(amount)?
                    .div(10_u32)
                    .saturating_mul(U256::from(12_u32)); // 20% of headroom
                let float_wallets: f32 = number_of_wallets.into();
                let amount_of_bt_to_transfer_for_each: f32 = amount / float_wallets;
                let amount_of_bt_to_withdraw: f32 = amount;
                // Here we are assuming that the base token has 18 decimals
                let parsed_amount_of_bt_to_transfer_for_each =
                    parse_ether(amount_of_bt_to_transfer_for_each)?;

                // Begin Display L1 Balance and BaseToken Addr
                println!("{}", "#".repeat(64));
                println!(
                    "{}: {base_token_address:?}",
                    "Base Token Address".bold().green().on_black()
                );
                display_balance(None, &zk_wallet, true).await?;
                display_balance(Some(base_token_address), &zk_wallet, true).await?;

                let (l1_balance, _, token_symbol) = get_erc20_balance_decimals_symbol(
                    base_token_address,
                    zk_wallet.l1_address(),
                    &l1_provider,
                )
                .await?;

                // Here we are assuming that the base token has 18 decimals
                if parse_ether(l1_balance)?.le(&parsed_amount_to_deposit) {
                    let mint_amount = parsed_amount_to_deposit
                        .div(100_u32)
                        .saturating_mul(U256::from(120_u32));
                    let future_receipt = erc20_l1_mint(base_token_address, &zk_wallet, mint_amount);
                    let msg = format!(
                        "Not enough tokens... Minting {} {token_symbol}",
                        format_ether(mint_amount)
                    );
                    let mut spinner = Spinner::new(spinners::Dots, msg, Color::Blue);
                    let receipt = future_receipt.await?;
                    spinner.success("Tokens Minted!");
                    println!("Transaction Hash: {:?}", receipt.transaction_hash);
                }

                display_balance(Some(base_token_address), &zk_wallet, true).await?;
                println!("{}", "#".repeat(64));
                // End Display L1 Balance and BaseToken Addr

                let mut reruns = 0;
                let mut current_reruns: u32 = 1;
                let reruns_wanted = reruns_wanted.unwrap_or(1);
                let reruns_to_complete = if reruns_wanted == 0 { 1 } else { reruns_wanted };

                println!(
                    "Number of reruns {}",
                    if reruns_wanted == 0 {
                        "∞".to_owned().red()
                    } else {
                        reruns_wanted.to_string().red()
                    }
                );

                while reruns < reruns_to_complete {
                    println!(
                        "{} N: {}",
                        "Run".red().on_black(),
                        (current_reruns).to_string().yellow().on_black()
                    );
                    // Begin Deposit from rich wallet to rich wallet
                    let l2_balance = zk_wallet
                        .l2_provider()
                        .get_balance(zk_wallet.l2_address(), None)
                        .await?;
                    if l2_balance.le(&parsed_amount_to_deposit) {
                        println!("{}", "#".repeat(64));
                        println!(
                            "{} Deposit from {} wallet to {} wallet.",
                            "[L1->L2]".bold().bright_cyan().on_black(),
                            "rich".bold().red().on_black(),
                            "rich".bold().red().on_black()
                        );
                        display_balance(None, &zk_wallet, false).await?;
                        zk_wallet
                            .deposit_base_token(parsed_amount_to_deposit)
                            .await?;
                        display_balance(None, &zk_wallet, false).await?;
                        println!("{}", "#".repeat(64));
                    }
                    // End Deposit from rich wallet to rich wallet

                    // Begin Transfer from rich wallet to each wallet
                    println!("{}", "#".repeat(64));
                    println!(
                        "{} Transfer from {} wallet to {} wallet.",
                        "[L2->L2]".bold().bright_cyan().on_black(),
                        "rich".bold().red().on_black(),
                        "each".bold().blue().on_black()
                    );

                    let mut futures = Vec::new();
                    let mut nonce = l2_provider
                        .get_transaction_count(zk_wallet.l2_address(), None)
                        .await?;
                    for w in &wallets {
                        let transfer_future = future_transfer_base_token(
                            &zk_wallet,
                            w,
                            parsed_amount_of_bt_to_transfer_for_each,
                            Some(L2TxOverrides::new().nonce(nonce)),
                        );
                        futures.push(transfer_future);
                        nonce = nonce.saturating_add(U256::one());
                    }
                    println!(
                        "{}",
                        "Waiting for all transactions to finish".yellow().on_black()
                    );
                    for f in futures {
                        f.await?;
                    }
                    println!("{}", "#".repeat(64));
                    // End Transfer from rich wallet to each wallet

                    // Begin Transfer from each wallet to rich wallet
                    println!("{}", "#".repeat(64));
                    println!(
                        "{} Transfer from {} wallet to {} wallet.",
                        "[L1->L2]".bold().bright_cyan().on_black(),
                        "each".bold().blue().on_black(),
                        "rich".bold().red().on_black()
                    );
                    let mut futures = Vec::new();
                    for w in &wallets {
                        let transfer_future = future_transfer_base_token_back(w, &zk_wallet);
                        futures.push(transfer_future);
                    }
                    println!(
                        "{}",
                        "Waiting for all transactions to finish".yellow().on_black()
                    );
                    for f in futures {
                        f.await?;
                    }
                    println!("{}", "#".repeat(64));
                    // End Transfer from each wallet to rich wallet

                    // Begin Withdrawal
                    println!("{}", "#".repeat(64));
                    println!(
                        "{} Withdraw basetoken from {} wallet.",
                        "[L2->L1]".bold().bright_cyan().on_black(),
                        "rich".bold().red().on_black(),
                    );
                    display_balance(None, &zk_wallet, false).await?;
                    let withdraw_hash = zk_wallet
                        .withdraw_base_token(parse_ether(amount_of_bt_to_withdraw.to_string())?)
                        .await?;
                    println!("Withdraw hash: {withdraw_hash:?}");
                    display_balance(None, &zk_wallet, false).await?;
                    let base_token_address = Some(l2_provider.get_base_token_l1_address().await?);
                    display_balance(base_token_address, &zk_wallet, true).await?;
                    println!("finalize withdrawal");
                    wait_for_finalize_withdrawal(withdraw_hash, &l2_provider).await;
                    zk_wallet.finalize_withdraw(withdraw_hash).await?;
                    display_balance(base_token_address, &zk_wallet, true).await?;
                    println!("{}", "#".repeat(64));
                    // End Withdrawal
                    if reruns_wanted != 0 {
                        reruns += 1;
                    }
                    current_reruns += 1;
                }
                Ok(())
            }
        }
    }
}

async fn future_transfer_base_token(
    from_wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    to_wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    parsed_amount: U256,
    overrides: Option<L2TxOverrides>,
) -> eyre::Result<()> {
    display_balance(None, to_wallet, false).await?;

    let transfer_hash = from_wallet
        .transfer_base_token(parsed_amount, to_wallet.l2_address(), overrides)
        .await?;

    println!("Transfer hash: {transfer_hash:?}");

    display_balance(None, to_wallet, false).await?;

    Ok(())
}

async fn future_transfer_base_token_back(
    from_wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    to_wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
) -> eyre::Result<()> {
    display_balance(None, from_wallet, false).await?;
    display_balance(None, to_wallet, false).await?;
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
    display_balance(None, from_wallet, false).await?;
    display_balance(None, to_wallet, false).await?;
    Ok(())
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
