use crate::config::ZKSyncConfig;
use crate::utils::balance::{display_balance, get_erc20_balance_decimals_symbol};
use crate::utils::test::*;
use crate::utils::wallet::*;
use clap::Subcommand;
use colored::*;
use spinoff::{spinners, Color, Spinner};
use std::ops::Div;
use zksync_ethers_rs::core::utils::format_ether;
use zksync_ethers_rs::{
    core::utils::parse_ether,
    providers::Middleware,
    types::{L2TxOverrides, U256},
    wait_for_finalize_withdrawal, ZKMiddleware,
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
        let (zk_wallet, l1_provider, l2_provider) = get_wallet_l1_l2_providers(cfg)?;
        let base_token_address = l2_provider.get_base_token_l1_address().await?;

        match self {
            Command::LoadTest {
                number_of_wallets,
                amount,
                reruns_wanted,
            } => {
                let wallets =
                    get_n_random_wallets(number_of_wallets, &l1_provider, &l2_provider).await?;
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
                display_balance(None, &zk_wallet, true, false).await?;
                display_balance(Some(base_token_address), &zk_wallet, true, false).await?;

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
                    display_balance(Some(base_token_address), &zk_wallet, true, false).await?;
                    println!("Transaction Hash: {:?}", receipt.transaction_hash);
                }

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
                        display_balance(None, &zk_wallet, false, true).await?;
                        zk_wallet
                            .deposit_base_token(parsed_amount_to_deposit)
                            .await?;
                        display_balance(None, &zk_wallet, false, true).await?;
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
                    display_balance(None, &zk_wallet, true, true).await?;
                    let withdraw_hash = zk_wallet
                        .withdraw_base_token(parse_ether(amount_of_bt_to_withdraw.to_string())?)
                        .await?;
                    println!("Withdraw hash: {withdraw_hash:?}");
                    let base_token_address = Some(l2_provider.get_base_token_l1_address().await?);
                    println!("finalize withdrawal");
                    wait_for_finalize_withdrawal(withdraw_hash, &l2_provider).await;
                    zk_wallet.finalize_withdraw(withdraw_hash).await?;
                    display_balance(base_token_address, &zk_wallet, true, true).await?;
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
