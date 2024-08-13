use crate::config::ZKSyncConfig;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

pub(crate) async fn run(config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(config.l2_rpc_url)?;
    let bridge_contracts = provider.get_bridge_contracts().await?;
    if let Some(l1_shared_bridge) = bridge_contracts.l1_shared_default_bridge {
        println!("L1 Shared Bridge: {l1_shared_bridge:#?}");
    } else {
        println!("L1 Shared Bridge: Not set");
    }
    if let Some(l1_erc20_bridge) = bridge_contracts.l1_erc20_default_bridge {
        println!("L1 ERC20 Bridge: {l1_erc20_bridge:#?}");
    } else {
        println!("L1 ERC20 Bridge: Not set");
    }
    if let Some(l1_weth_bridge) = bridge_contracts.l1_weth_bridge {
        println!("L1 WETH Bridge: {l1_weth_bridge:#?}");
    } else {
        println!("L1 WETH Bridge: Not set");
    }
    if let Some(l2_shared_bridge) = bridge_contracts.l2_shared_default_bridge {
        println!("L2 Shared Bridge: {l2_shared_bridge:#?}");
    } else {
        println!("L2 Shared Bridge: Not set");
    }
    if let Some(l2_erc20_bridge) = bridge_contracts.l2_erc20_default_bridge {
        println!("L2 ERC20 Bridge: {l2_erc20_bridge:#?}");
    } else {
        println!("L2 ERC20 Bridge: Not set");
    }
    if let Some(l2_weth_bridge) = bridge_contracts.l2_weth_bridge {
        println!("L2 WETH Bridge: {l2_weth_bridge:#?}");
    } else {
        println!("L2 WETH Bridge: Not set");
    }
    Ok(())
}
