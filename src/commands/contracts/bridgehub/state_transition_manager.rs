use clap::Args as ClapArgs;
use zksync_ethers_rs::{
    contracts::bridgehub::Bridgehub,
    providers::Middleware,
    types::{Address, U256},
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(short, long, value_parser = U256::from_dec_str)]
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
