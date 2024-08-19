use zksync_ethers_rs::{contracts::bridgehub::Bridgehub, providers::Middleware, types::Address};

pub(crate) async fn run(bridgehub: Bridgehub<impl Middleware + 'static>) -> eyre::Result<()> {
    let bridgehub_admin: Address = bridgehub.admin().call().await?;
    println!("Bridgehub admin: {bridgehub_admin:?}");
    Ok(())
}
