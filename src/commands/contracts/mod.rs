use clap::Subcommand;

use crate::config::ZKSyncConfig;

pub(crate) mod bridgehub;
pub(crate) mod governance;
pub(crate) mod state_transition_manager;

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(
        subcommand,
        about = "Bridgehub contract interaction commands.",
        visible_alias = "bh"
    )]
    Bridgehub(bridgehub::Command),
    #[clap(
        subcommand,
        about = "Governance contract interaction commands.",
        visible_alias = "g"
    )]
    Governance(governance::Command),
    #[clap(
        subcommand,
        about = "Hyperchain contract interaction commands.",
        visible_alias = "h"
    )]
    Hyperchain,
    #[clap(
        subcommand,
        about = "L1SharedBridge contract interaction commands.",
        visible_alias = "l1sb"
    )]
    L1SharedBridge,
    #[clap(
        subcommand,
        about = "StateTransitionManager contract interaction commands.",
        visible_alias = "stm"
    )]
    StateTransitionManager(state_transition_manager::Command),
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        match self {
            Command::Bridgehub(cmd) => cmd.run(cfg).await?,
            Command::Governance(cmd) => cmd.run(cfg).await?,
            Command::Hyperchain => todo!(),
            Command::L1SharedBridge => todo!(),
            Command::StateTransitionManager(cmd) => cmd.run(cfg).await?,
        };

        Ok(())
    }
}
