use clap::Subcommand;

use crate::config::ZKSyncConfig;

pub(crate) mod bridgehub;
pub(crate) mod governance;
pub(crate) mod hyperchain;
pub(crate) mod l1_shared_bridge;
pub(crate) mod state_transition_manager;

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(subcommand, about = "Bridgehub contract interaction commands.")]
    BridgeHub(bridgehub::Command),
    #[clap(subcommand, about = "Governance contract interaction commands.")]
    Governance(governance::Command),
    #[clap(subcommand, about = "Hyperchain contract interaction commands.")]
    Hyperchain(hyperchain::Command),
    #[clap(subcommand, about = "L1SharedBridge contract interaction commands.")]
    L1SharedBridge(l1_shared_bridge::Command),
    #[clap(
        subcommand,
        about = "StateTransitionManager contract interaction commands."
    )]
    StateTransitionManager(state_transition_manager::Command),
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    match cmd {
        Command::BridgeHub(cmd) => bridgehub::start(cmd, cfg).await?,
        Command::Governance(cmd) => governance::start(cmd, cfg).await?,
        Command::Hyperchain(_cmd) => todo!(),
        Command::L1SharedBridge(_cmd) => todo!(),
        Command::StateTransitionManager(_cmd) => todo!(),
    };

    Ok(())
}
