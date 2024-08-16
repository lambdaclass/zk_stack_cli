use clap::Args as ClapArgs;
use zksync_ethers_rs::{abi::Hash, contracts::governance::Governance, providers::Middleware};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(short = 'o', long, index = 0)]
    pub operation_id: Hash,
}

pub(crate) async fn run(
    args: Args,
    governance: Governance<impl Middleware + 'static>,
) -> eyre::Result<()> {
    let operation_state: OperationState = governance
        .get_operation_state(args.operation_id.into())
        .call()
        .await?
        .into();
    println!("{operation_state:?}");
    Ok(())
}

#[derive(Debug)]
enum OperationState {
    Unset,
    Waiting,
    Ready,
    Done,
}

impl From<u8> for OperationState {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unset,
            1 => Self::Waiting,
            2 => Self::Ready,
            3 => Self::Done,
            _ => unreachable!(),
        }
    }
}
