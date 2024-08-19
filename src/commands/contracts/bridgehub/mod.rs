use crate::{commands::utils::try_bridgehub_from_config, config::ZKSyncConfig};
use clap::Subcommand;

pub(crate) mod accept_admin;
pub(crate) mod admin;
pub(crate) mod base_token;
pub(crate) mod get_hyperchain;
pub(crate) mod set_pending_admin;
pub(crate) mod state_transition_manager;

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "Get the StateTransitionManager contract address of a chain.")]
    StateTransitionManager(state_transition_manager::Args),
    #[clap(about = "Get the base token contract of a chain.")]
    BaseToken(base_token::Args),
    #[clap(about = "Get the bridge contract admin address.")]
    Admin,
    #[clap(
        about = "Set a new admin of the Bridgehub. Only the Bridgehub owner or the current admin can do this."
    )]
    SetPendingAdmin(set_pending_admin::Args),
    #[clap(about = "Accept the admin of the Bridgehub. Only the pending admin can do this.")]
    AcceptAdmin(accept_admin::Args),
    #[clap(about = "Get the Hyperchain contract address of a chain.")]
    GetHyperchain(get_hyperchain::Args),
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let bridgehub = try_bridgehub_from_config(&cfg).await?;
    match cmd {
        Command::StateTransitionManager(args) => {
            state_transition_manager::run(args, bridgehub).await?
        }
        Command::BaseToken(args) => base_token::run(args, bridgehub).await?,
        Command::Admin => admin::run(bridgehub).await?,
        Command::SetPendingAdmin(args) => set_pending_admin::run(args, bridgehub, cfg).await?,
        Command::AcceptAdmin(args) => accept_admin::run(args, bridgehub, cfg).await?,
        Command::GetHyperchain(args) => get_hyperchain::run(args, bridgehub).await?,
    };
    Ok(())
}
