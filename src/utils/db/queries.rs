use crate::utils::db::{types::ProverJobFriInfo, CURRENT_MAX_ATTEMPTS};
use eyre::ContextCompat;
use sqlx::{pool::PoolConnection, postgres::PgRow, Executor, FromRow, Postgres, Row};
use std::str::FromStr;
use zksync_ethers_rs::types::zksync::{
    basic_fri_types::AggregationRound,
    prover_dal::{ProofCompressionJobStatus, WitnessJobStatus},
    L1BatchNumber,
};

pub async fn get_batch_proofs_stuck_at_wg_round<WG>(
    aggregation_round: AggregationRound,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Vec<WG>>
where
    WG: for<'row> FromRow<'row, PgRow>,
{
    let table = input_table_name_for(aggregation_round);
    let query = format!(
        "
        SELECT *
        FROM {table}
        WHERE
            attempts = {CURRENT_MAX_ATTEMPTS}
            AND status != 'success'
        "
    );
    prover_db
        .fetch_all(query.as_str())
        .await?
        .iter()
        .map(WG::from_row)
        .collect::<Result<Vec<WG>, _>>()
        .map_err(Into::into)
}

pub async fn get_batch_proofs_stuck_at_prover_in_agg_round(
    prover_db: &mut PoolConnection<Postgres>,
    aggregation_round: AggregationRound,
) -> eyre::Result<Vec<ProverJobFriInfo>> {
    let query = format!(
        "
        SELECT *
        FROM prover_jobs_fri
        WHERE
            attempts = {CURRENT_MAX_ATTEMPTS}
            AND status != 'success'
            AND aggregation_round = {}
        ",
        aggregation_round as u8
    );
    prover_db
        .fetch_all(query.as_str())
        .await?
        .iter()
        .map(ProverJobFriInfo::from_row)
        .collect::<Result<Vec<ProverJobFriInfo>, _>>()
        .map_err(Into::into)
}

pub async fn get_compressor_job_status(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Option<ProofCompressionJobStatus>> {
    let query = format!(
        "
        SELECT status
        FROM prover_jobs_fri
        WHERE
            l1_batch_number = {}
            AND status = 'sent_to_server'
        ",
        l1_batch_number.0,
    );
    prover_db
        .fetch_optional(query.as_str())
        .await?
        .map(|row| {
            let raw_status: String = row.get("status");
            ProofCompressionJobStatus::from_str(raw_status.as_str()).ok()
        })
        .context("Failed to get compressor job status")
}

pub async fn restart_batch_proof(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    delete_batch_proof_compression_data(l1_batch_number, prover_db).await?;
    delete_batch_witness_generation_data(AggregationRound::Scheduler, l1_batch_number, prover_db)
        .await?;
    delete_batch_witness_generation_data(
        AggregationRound::RecursionTip,
        l1_batch_number,
        prover_db,
    )
    .await?;
    delete_batch_witness_generation_data(
        AggregationRound::NodeAggregation,
        l1_batch_number,
        prover_db,
    )
    .await?;
    delete_batch_witness_generation_data(
        AggregationRound::LeafAggregation,
        l1_batch_number,
        prover_db,
    )
    .await?;
    delete_batch_proof_prover_data(l1_batch_number, prover_db).await?;
    set_basic_witness_generator_job_status(l1_batch_number, WitnessJobStatus::Queued, prover_db)
        .await
}

pub async fn set_basic_witness_generator_job_status(
    l1_batch_number: L1BatchNumber,
    status: WitnessJobStatus,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    let query = format!(
        "
        UPDATE witness_inputs_fri
        SET status = '{status}'
        WHERE
            l1_batch_number = {l1_batch_number}
        ",
        status = status,
        l1_batch_number = l1_batch_number.0
    );
    prover_db.execute(query.as_str()).await?;
    Ok(())
}

pub async fn delete_batch_proof_compression_data(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    delete_batch_data_from_table(l1_batch_number, "proof_compression_jobs_fri", prover_db).await
}

pub async fn delete_batch_witness_generation_data(
    aggregation_round: AggregationRound,
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    delete_batch_data_from_table(
        l1_batch_number,
        input_table_name_for(aggregation_round),
        prover_db,
    )
    .await
}

pub async fn delete_batch_proof_prover_data(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    delete_batch_data_from_table(l1_batch_number, "prover_jobs_fri", prover_db).await
}

async fn delete_batch_data_from_table(
    l1_batch_number: L1BatchNumber,
    table: &str,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    let query = format!(
        "
        DELETE FROM {table}
        WHERE
            l1_batch_number = {l1_batch_number}
        ",
        table = table,
        l1_batch_number = l1_batch_number.0
    );
    prover_db.execute(query.as_str()).await?;
    Ok(())
}

fn input_table_name_for(aggregation_round: AggregationRound) -> &'static str {
    match aggregation_round {
        AggregationRound::BasicCircuits => "witness_inputs_fri",
        AggregationRound::LeafAggregation => "leaf_aggregation_witness_jobs_fri",
        AggregationRound::NodeAggregation => "node_aggregation_witness_jobs_fri",
        AggregationRound::RecursionTip => "recursion_tip_witness_jobs_fri",
        AggregationRound::Scheduler => "scheduler_witness_jobs_fri",
    }
}
