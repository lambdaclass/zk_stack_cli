use crate::{config::ZKSyncConfig, utils::wallet::get_wallet_l1_l2_providers};
use clap::Subcommand;
use colored::Colorize;
use spinoff::{spinners, Color, Spinner};
use std::path::PathBuf;
use zksync_ethers_rs::{
    types::zksync::{api::L1BatchDetails, inputs::WitnessInputData, L1BatchNumber},
    ZKMiddleware,
};

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(
        about = "Prover - Debug Witness Inputs",
        visible_alias = "debug-proof-gen-data"
    )]
    DebugWitnessInputs {
        file_path: PathBuf,
        #[arg(long, default_value = "false", requires = "file_path")]
        vm_run_data: bool,
        #[arg(long, default_value = "false", requires = "file_path")]
        merkle_paths: bool,
        #[arg(long, default_value = "false", requires = "file_path")]
        previous_batch_metadata: bool,
        #[arg(long, default_value = "false", requires = "file_path")]
        eip_4844_blobs: bool,
    },
    #[clap(
        about = "Prover - Batch Details. It gets the proof-time of the specified batch.",
        visible_alias = "batch-details"
    )]
    BatchDetails {
        #[clap(short = 'n', num_args = 1..)]
        batches: Option<Vec<L1BatchNumber>>,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        match self {
            Command::DebugWitnessInputs {
                file_path,
                vm_run_data,
                merkle_paths,
                previous_batch_metadata,
                eip_4844_blobs,
            } => {
                let witness_inputs_bytes = std::fs::read(file_path)?;
                let witness_input_data: WitnessInputData =
                    bincode::deserialize(&witness_inputs_bytes)?;
                if vm_run_data && merkle_paths && previous_batch_metadata && eip_4844_blobs {
                    println!("{witness_input_data:?}");
                } else {
                    if vm_run_data {
                        println!("{:?}", witness_input_data.vm_run_data);
                    }
                    if merkle_paths {
                        println!("{:?}", witness_input_data.merkle_paths);
                    }
                    if previous_batch_metadata {
                        println!("{:?}", witness_input_data.previous_batch_metadata);
                    }
                    if eip_4844_blobs {
                        println!("{:?}", witness_input_data.eip_4844_blobs);
                    }
                }
            }
            Command::BatchDetails { batches } => {
                let (_, _, l2_provider) = get_wallet_l1_l2_providers(cfg)?;

                let current_batch = l2_provider.get_l1_batch_number().await?.as_u32().into();

                let batches_vec = if let Some(batches_vec) = batches {
                    batches_vec
                } else {
                    vec![current_batch]
                };

                let msg = if batches_vec.len() > 1 {
                    "Fetching Batches' Data"
                } else {
                    "Fetching Batch's Data"
                };

                let mut spinner = Spinner::new(spinners::Dots, msg, Color::Blue);

                let mut batches_details = Vec::new();
                for batch in batches_vec {
                    if batch.0 > current_batch.0 {
                        println!("Batch doesn't exist, Current batch: {}", current_batch.0);
                        break;
                    }
                    batches_details.push(l2_provider.get_l1_batch_details(batch.0).await?);
                }
                spinner.success("Success");
                display_batches_proof_time_from_details(batches_details);
            }
        }
        Ok(())
    }
}

fn display_batches_proof_time_from_details(batches_details: Vec<L1BatchDetails>) {
    for batch_details in batches_details {
        println!(
            "{} {} {}",
            "=".repeat(8),
            format!("Batch {:0>5} Status", batch_details.number.0)
                .bold()
                .bright_cyan()
                .on_black(),
            "=".repeat(8)
        );
        if let Some(committed_at) = batch_details.base.committed_at {
            println!("Committed At: {committed_at}");
        }
        if let Some(commit_tx_hash) = batch_details.base.commit_tx_hash {
            println!("Commit Tx Hash: {commit_tx_hash:?}");
        }

        if let Some(proven_at) = batch_details.base.proven_at {
            println!("Proven At: {proven_at}");
        }
        if let Some(prove_tx_hash) = batch_details.base.prove_tx_hash {
            println!("Proven Tx Hash: {prove_tx_hash:?}");
        }
    }
}
