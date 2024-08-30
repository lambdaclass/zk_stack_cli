use crate::utils::db::types::ProverJobFriInfo;
use crate::utils::db::types::{
    BasicWitnessGeneratorJobInfo, LeafWitnessGeneratorJobInfo, NodeWitnessGeneratorJobInfo,
    SchedulerWitnessGeneratorJobInfo,
};
use sqlx::{pool::PoolConnection, Executor, FromRow, Postgres};
use zksync_ethers_rs::types::zksync::prover_dal::ProofCompressionJobInfo;
use zksync_ethers_rs::types::zksync::{basic_fri_types::AggregationRound, L1BatchNumber};

use super::types::RecursionTipWitnessGeneratorJobInfo;

async fn get_prover_jobs_info_for_batch(
    _batch_number: L1BatchNumber,
    _aggregation_round: AggregationRound,
    _prover_db: &mut PoolConnection<Postgres>,
) -> Vec<ProverJobFriInfo> {
    todo!()
}

async fn get_proof_basic_witness_generator_into_for_batch(
    _batch_number: L1BatchNumber,
    _prover_db: &mut PoolConnection<Postgres>,
) -> Option<BasicWitnessGeneratorJobInfo> {
    todo!()
}

// https://github.com/matter-labs/zksync-era/blob/6d18061df4a18803d3c6377305ef711ce60317e1/prover/crates/lib/prover_dal/src/fri_witness_generator_dal.rs#L1489
async fn get_proof_leaf_witness_generator_info_for_batch(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Vec<LeafWitnessGeneratorJobInfo>> {
    let query = format!(
        "
        SELECT *
        FROM leaf_aggregation_witness_jobs_fri
        WHERE
            l1_batch_number = {}
        ",
        l1_batch_number.0,
    );

    prover_db
        .fetch_all(query.as_str())
        .await?
        .iter()
        .map(LeafWitnessGeneratorJobInfo::from_row)
        .collect::<Result<Vec<LeafWitnessGeneratorJobInfo>, _>>()
        .map_err(Into::into)
}

async fn get_proof_node_witness_generator_info_for_batch(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Vec<NodeWitnessGeneratorJobInfo>> {
    let query = format!(
        "
        SELECT *
        FROM node_aggregation_witness_jobs_fri
        WHERE
            l1_batch_number = {}
        ",
        l1_batch_number.0,
    );

    prover_db
        .fetch_all(query.as_str())
        .await?
        .iter()
        .map(NodeWitnessGeneratorJobInfo::from_row)
        .collect::<Result<Vec<NodeWitnessGeneratorJobInfo>, _>>()
        .map_err(Into::into)
}

async fn get_proof_recursion_tip_witness_generator_info_for_batch(
    _batch_number: L1BatchNumber,
    _prover_db: &mut PoolConnection<Postgres>,
) -> Option<RecursionTipWitnessGeneratorJobInfo> {
    todo!()
}

async fn get_proof_scheduler_witness_generator_info_for_batch(
    _batch_number: L1BatchNumber,
    _prover_db: &mut PoolConnection<Postgres>,
) -> Option<SchedulerWitnessGeneratorJobInfo> {
    todo!()
}

async fn get_proof_compression_job_info_for_batch(
    _batch_number: L1BatchNumber,
    _prover_db: &mut PoolConnection<Postgres>,
) -> Option<ProofCompressionJobInfo> {
    todo!()
}
