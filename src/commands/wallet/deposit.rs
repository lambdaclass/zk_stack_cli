use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    middleware::SignerMiddleware,
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
    zk_wallet::ZKWallet,
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "amount", value_parser=U256::from_dec_str)]
    pub amount: U256,
    #[clap(long = "token")]
    pub token_address: Option<Address>,
    #[clap(long = "from")]
    pub from: Option<LocalWallet>,
    #[clap(long = "to")]
    pub to: Option<Address>,
    #[clap(long, required = false)]
    explorer_url: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let wallet_config = cfg.wallet.context("Wallet config missing")?;

    let l1_provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
    let wallet = args
        .from
        .unwrap_or(wallet_config.private_key.parse()?)
        .with_chain_id(l1_chain_id);
    let l1_signer = SignerMiddleware::new(l1_provider, wallet.clone());

    let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let l1_chain_id = l2_provider.get_chainid().await?.as_u64();
    let wallet = wallet.with_chain_id(l1_chain_id);
    let l2_signer = SignerMiddleware::new(l2_provider, wallet);

    let zk_wallet = ZKWallet::new(l1_signer, l2_signer);

    let amount = U256::from(args.amount);
    let deposit_hash = match (args.to, args.token_address) {
        (None, None) => zk_wallet.deposit_base_token(amount).await?,
        (None, Some(token)) => zk_wallet.deposit_erc20(amount, token).await?,
        (Some(to), None) => zk_wallet.deposit_base_token_to(amount, to).await?,
        (Some(to), Some(token)) => zk_wallet.deposit_erc20_to(amount, token, to).await?,
    };

    if args.explorer_url {
        let url = cfg
            .network
            .l1_explorer_url
            .context("L1 Explorer URL missing in config")?;
        println!("Deposit: {url}/tx/{deposit_hash:?}");
    } else {
        println!("Deposit hash: {deposit_hash:?}");
    }

    Ok(())
}
