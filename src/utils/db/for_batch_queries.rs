use crate::utils::db::types::{
    BasicWitnessGeneratorJobInfo, LeafWitnessGeneratorJobInfo, NodeWitnessGeneratorJobInfo,
    ProofCompressionJobInfo, ProverJobFriInfo, SchedulerWitnessGeneratorJobInfo,
};
use eyre::ContextCompat;
use sqlx::{pool::PoolConnection, Executor, FromRow, Postgres};
use zksync_ethers_rs::types::zksync::{basic_fri_types::AggregationRound, L1BatchNumber};

use super::types::RecursionTipWitnessGeneratorJobInfo;

#[allow(clippy::as_conversions, reason = "AggregationRound is an enum of u8s")]
pub(crate) async fn get_prover_jobs_info_for_batch(
    l1_batch_number: L1BatchNumber,
    aggregation_round: AggregationRound,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Vec<ProverJobFriInfo>> {
    let query = format!(
        "
        SELECT *
        FROM prover_jobs_fri
        WHERE
            l1_batch_number = {}
            AND aggregation_round = {}
        ",
        l1_batch_number.0, aggregation_round as u8
    );

    prover_db
        .fetch_all(query.as_str())
        .await?
        .iter()
        .map(ProverJobFriInfo::from_row)
        .collect::<Result<Vec<ProverJobFriInfo>, _>>()
        .map_err(Into::into)
}

pub(crate) async fn get_proof_basic_witness_generator_info_for_batch(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Option<BasicWitnessGeneratorJobInfo>> {
    let query = format!(
        "
        SELECT *
        FROM witness_inputs_fri
        WHERE
            l1_batch_number = {}
        ",
        l1_batch_number.0,
    );

    let row = prover_db
        .fetch_optional(query.as_str())
        .await?
        .context("Parsing Row")?;

    Ok(Some(BasicWitnessGeneratorJobInfo::from_row(&row)?))
}

pub(crate) async fn get_proof_leaf_witness_generator_info_for_batch(
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

pub(crate) async fn get_proof_node_witness_generator_info_for_batch(
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

pub(crate) async fn get_proof_recursion_tip_witness_generator_info_for_batch(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Option<RecursionTipWitnessGeneratorJobInfo>> {
    let query = format!(
        "
        SELECT *
        FROM recursion_tip_witness_jobs_fri
        WHERE
            l1_batch_number = {}
        ",
        l1_batch_number.0,
    );

    let row = prover_db
        .fetch_optional(query.as_str())
        .await?
        .context("Parsing Row")?;

    Ok(Some(RecursionTipWitnessGeneratorJobInfo::from_row(&row)?))
}

pub(crate) async fn get_proof_scheduler_witness_generator_info_for_batch(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Option<SchedulerWitnessGeneratorJobInfo>> {
    let query = format!(
        "
        SELECT *
        FROM scheduler_witness_jobs_fri
        WHERE
            l1_batch_number = {}
        ",
        l1_batch_number.0,
    );

    let row = prover_db
        .fetch_optional(query.as_str())
        .await?
        .context("Parsing Row")?;

    Ok(Some(SchedulerWitnessGeneratorJobInfo::from_row(&row)?))
}

pub(crate) async fn get_proof_compression_job_info_for_batch(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Option<ProofCompressionJobInfo>> {
    let query = format!(
        "
        SELECT *
        FROM proof_compression_jobs_fri
        WHERE
            l1_batch_number = {}
        ",
        l1_batch_number.0,
    );

    let row = prover_db
        .fetch_optional(query.as_str())
        .await?
        .context("Parsing Row")?;

    Ok(Some(ProofCompressionJobInfo::from_row(&row)?))
}
