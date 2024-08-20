use clap::Parser;
use zksync_ethers_rs::{
    contracts::bridgehub::Bridgehub,
    providers::Middleware,
    types::{Address, U256},
};

#[derive(Parser, PartialEq)]
pub(crate) struct Args {
    #[clap(value_parser = U256::from_dec_str)]
    pub chain_id: U256,
}

pub(crate) async fn run(
    args: Args,
    bridgehub: Bridgehub<impl Middleware + 'static>,
) -> eyre::Result<()> {
    let state_transition_manager: Address = bridgehub
        .state_transition_manager(args.chain_id)
        .call()
        .await?;
    println!(
        "STM for chain ID {:?}: {state_transition_manager:?}",
        args.chain_id
    );
    Ok(())
}
