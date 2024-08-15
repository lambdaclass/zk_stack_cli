use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use std::collections::HashMap;
use zksync_ethers_rs::{
    core::utils::format_ether, providers::Provider, types::Address, ZKMiddleware,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "address")]
    pub account_address: Address,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    // Retrieving the L2 balances, the token addresses will not be usable on L1
    // A way to transform the token addresses from L2 to L1 may be needed
    let all_account_balances = provider
        .get_all_account_balances(args.account_address)
        .await?;
    let mut all_account_parsed_balances: HashMap<Address, String> = HashMap::new();
    for (k, v) in all_account_balances {
        // Assuming all tokens have 18 Decimals
        // To have display the balance better use the address provided with this cmd
        // And use the balance cmd with the token address
        let v = format_ether(v);
        all_account_parsed_balances.insert(k, v);
    }
    println!("{all_account_parsed_balances:#?}");
    Ok(())
}
