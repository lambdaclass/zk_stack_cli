use super::db::{
    for_batch_queries::*,
    types::{
        BasicWitnessGeneratorJobInfo, LeafWitnessGeneratorJobInfo, NodeWitnessGeneratorJobInfo,
        ProofCompressionJobInfo, ProverJobFriInfo, RecursionTipWitnessGeneratorJobInfo,
        SchedulerWitnessGeneratorJobInfo,
    },
};
use sqlx::{pool::PoolConnection, Postgres};
use strum::{Display, EnumString};
use zksync_ethers_rs::types::zksync::{basic_fri_types::AggregationRound, L1BatchNumber};

// From: zksync-era:
// https://github.com/matter-labs/zksync-era/blob/main/prover/crates/bin/prover_cli/src/commands/status/utils.rs#L171
#[allow(clippy::large_enum_variant, reason = "strum")]
#[derive(EnumString, Clone, Display, Debug)]
pub enum StageInfo {
    #[strum(to_string = "Basic Witness Generator")]
    BasicWitnessGenerator {
        witness_generator_job_info: Option<BasicWitnessGeneratorJobInfo>,
        prover_jobs_info: Vec<ProverJobFriInfo>,
    },
    #[strum(to_string = "Leaf Witness Generator")]
    LeafWitnessGenerator {
        witness_generator_jobs_info: Vec<LeafWitnessGeneratorJobInfo>,
        prover_jobs_info: Vec<ProverJobFriInfo>,
    },
    #[strum(to_string = "Node Witness Generator")]
    NodeWitnessGenerator {
        witness_generator_jobs_info: Vec<NodeWitnessGeneratorJobInfo>,
        prover_jobs_info: Vec<ProverJobFriInfo>,
    },
    #[strum(to_string = "Recursion Tip")]
    RecursionTipWitnessGenerator(Option<RecursionTipWitnessGeneratorJobInfo>),
    #[strum(to_string = "Scheduler")]
    SchedulerWitnessGenerator(Option<SchedulerWitnessGeneratorJobInfo>),
    #[strum(to_string = "Compressor")]
    Compressor(Option<ProofCompressionJobInfo>),
}

#[derive(Debug)]
/// Represents the proving data of a l1_batch_number.
pub struct BatchData {
    /// The number of the l1_batch_number.
    pub batch_number: L1BatchNumber,
    /// The basic witness generator data.
    pub basic_witness_generator: StageInfo,
    /// The leaf witness generator data.
    pub leaf_witness_generator: StageInfo,
    /// The node witness generator data.
    pub node_witness_generator: StageInfo,
    /// The recursion tip data.
    pub recursion_tip_witness_generator: StageInfo,
    /// The scheduler data.
    pub scheduler_witness_generator: StageInfo,
    /// The compressor data.
    pub compressor: StageInfo,
}

pub async fn get_batches_data(
    batches: Vec<L1BatchNumber>,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Vec<BatchData>> {
    let mut batches_data = Vec::new();
    for l1_batch_number in batches {
        let current_batch_data = BatchData {
            batch_number: l1_batch_number,
            basic_witness_generator: StageInfo::BasicWitnessGenerator {
                witness_generator_job_info: get_proof_basic_witness_generator_info_for_batch(
                    l1_batch_number,
                    prover_db,
                )
                .await?,
                prover_jobs_info: get_prover_jobs_info_for_batch(
                    l1_batch_number,
                    AggregationRound::BasicCircuits,
                    prover_db,
                )
                .await?,
            },
            leaf_witness_generator: StageInfo::LeafWitnessGenerator {
                witness_generator_jobs_info: get_proof_leaf_witness_generator_info_for_batch(
                    l1_batch_number,
                    prover_db,
                )
                .await?,
                prover_jobs_info: get_prover_jobs_info_for_batch(
                    l1_batch_number,
                    AggregationRound::LeafAggregation,
                    prover_db,
                )
                .await?,
            },
            node_witness_generator: StageInfo::NodeWitnessGenerator {
                witness_generator_jobs_info: get_proof_node_witness_generator_info_for_batch(
                    l1_batch_number,
                    prover_db,
                )
                .await?,
                prover_jobs_info: get_prover_jobs_info_for_batch(
                    l1_batch_number,
                    AggregationRound::NodeAggregation,
                    prover_db,
                )
                .await?,
            },
            recursion_tip_witness_generator: StageInfo::RecursionTipWitnessGenerator(
                get_proof_recursion_tip_witness_generator_info_for_batch(
                    l1_batch_number,
                    prover_db,
                )
                .await?,
            ),
            scheduler_witness_generator: StageInfo::SchedulerWitnessGenerator(
                get_proof_scheduler_witness_generator_info_for_batch(l1_batch_number, prover_db)
                    .await?,
            ),
            compressor: StageInfo::Compressor(
                get_proof_compression_job_info_for_batch(l1_batch_number, prover_db).await?,
            ),
        };
        println!("here");
        batches_data.push(current_batch_data);
    }
    Ok(batches_data)
}
