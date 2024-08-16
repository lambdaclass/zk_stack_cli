use clap::Subcommand;

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "See if the StateTransitionManager is registered.")]
    StateTransitionManagerIsRegistered,
    #[clap(about = "See if some arbitrary base token is registered.")]
    TokenIsRegistered,
    #[clap(about = "Get the StateTransitionManager contract address of a chain.")]
    StateTransitionManager,
    #[clap(about = "Get the base token contract of a chain.")]
    BaseToken,
    #[clap(about = "Get the bridge contract admin address.")]
    Admin,
    #[clap(
        about = "Set a new admin of the Bridgehub. Only the Bridgehub owner or the current admin can do this."
    )]
    SetPendingAdmin,
    #[clap(about = "Accept the admin of the Bridgehub. Only the pending admin can do this.")]
    AcceptAdmin,
    #[clap(about = "Get the Hyperchain contract address of a chain.")]
    GetHyperchain,
    #[clap(
        about = "Registers a new StateTransitionManager contract. Only the Bridgehub owner can do this."
    )]
    AddStateTransitionManager,
    #[clap(
        about = "Unregister a StateTransitionManager contract. Only the Bridgehub owner can do this."
    )]
    RemoveStateTransitionManager,
    #[clap(about = "Registers a new token. Only the Bridgehub owner can do this.")]
    AddToken,
    #[clap(about = "Sets the shared bridge. Only the Bridgehub owner can do this.")]
    SetSharedBridge,
    #[clap(
        about = "Pauses the Bridgehub. Only the Bridgehub owner can do this. This \"disables\" the L1->L2 communication and the creation of new chains."
    )]
    Pause,
    #[clap(
        about = "Unpauses the Bridgehub. Only the Bridgehub owner can do this. This \"re-enables\" the L1->L2 communication and the creation of new chains."
    )]
    Unpause,
}
