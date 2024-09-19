use crate::utils::db::{types::ProverJobFriInfo, CURRENT_MAX_ATTEMPTS};
use eyre::ContextCompat;
use sqlx::{pool::PoolConnection, postgres::PgRow, Executor, FromRow, Postgres, Row};
use std::str::FromStr;
use zksync_ethers_rs::{
    abi::Hash,
    types::zksync::{
        basic_fri_types::AggregationRound,
        protocol_version::VersionPatch,
        prover_dal::{ProofCompressionJobStatus, ProofGenerationTime, WitnessJobStatus},
        L1BatchNumber, ProtocolVersionId,
    },
};

use super::types::proof_generation_time_from_row;

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

#[allow(clippy::as_conversions, reason = "AggregationRound is an enum of u8s")]
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
        FROM proof_compression_jobs_fri
        WHERE
            l1_batch_number = {}
            AND status = 'sent_to_server'
        ",
        l1_batch_number.0,
    );
    let row = prover_db
        .fetch_optional(query.as_str())
        .await?
        .context("Parsing Row")?;
    let raw_status: String = row.get("status");
    Ok(Some(ProofCompressionJobStatus::from_str(
        raw_status.as_str(),
    )?))
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

#[allow(clippy::as_conversions, reason = "Allow as for ProtocolVersionId")]
pub async fn insert_witness_inputs(
    l1_batch_number: L1BatchNumber,
    witness_inputs_blob_url: &str,
    protocol_version: ProtocolVersionId,
    protocol_version_patch: VersionPatch,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    let query = format!(
        "
        INSERT INTO
            witness_inputs_fri (
                l1_batch_number,
                witness_inputs_blob_url,
                protocol_version,
                status,
                created_at,
                updated_at,
                protocol_version_patch
            )
        VALUES
            ({}, '{}', {}, 'queued', NOW(), NOW(), {})
        ON CONFLICT (l1_batch_number) DO NOTHING
        ",
        l1_batch_number.0, witness_inputs_blob_url, protocol_version as u16, protocol_version_patch
    );
    prover_db.execute(query.as_str()).await?;
    Ok(())
}

pub async fn get_basic_witness_job_status(
    l1_batch_number: L1BatchNumber,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Option<WitnessJobStatus>> {
    let query = format!(
        "
        SELECT status
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
    let raw_status: String = row.get("status");
    Ok(Some(WitnessJobStatus::from_str(raw_status.as_str())?))
}

#[allow(clippy::as_conversions, reason = "Allow as for ProtocolVersionId")]
pub async fn insert_prover_protocol_version(
    protocol_version: ProtocolVersionId,
    recursion_scheduler_level_vk_hash: Hash,
    recursion_node_level_vk_hash: Hash,
    recursion_leaf_level_vk_hash: Hash,
    recursion_circuits_set_vks_hash: Hash,
    protocol_version_patch: VersionPatch,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    let query = format!(
        "
        INSERT INTO
            prover_fri_protocol_versions (
                id,
                recursion_scheduler_level_vk_hash,
                recursion_node_level_vk_hash,
                recursion_leaf_level_vk_hash,
                recursion_circuits_set_vks_hash,
                protocol_version_patch,
                created_at
            )
        VALUES
            ({}, '\\x{:x}', '\\x{:x}', '\\x{:x}', '\\x{:x}', {}, NOW())
        ON CONFLICT (id, protocol_version_patch) DO UPDATE SET
            recursion_scheduler_level_vk_hash = EXCLUDED.recursion_scheduler_level_vk_hash,
            recursion_node_level_vk_hash = EXCLUDED.recursion_node_level_vk_hash,
            recursion_leaf_level_vk_hash = EXCLUDED.recursion_leaf_level_vk_hash,
            recursion_circuits_set_vks_hash = EXCLUDED.recursion_circuits_set_vks_hash
        ",
        protocol_version as u16,
        recursion_scheduler_level_vk_hash,
        recursion_node_level_vk_hash,
        recursion_leaf_level_vk_hash,
        recursion_circuits_set_vks_hash,
        protocol_version_patch
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

pub async fn get_proof_time(
    prover_db: &mut PoolConnection<Postgres>,
    l1_batch_number: Option<L1BatchNumber>,
    days: u32,
) -> eyre::Result<Vec<ProofGenerationTime>> {
    let query = match l1_batch_number {
        None => {
            format!(
                "
                SELECT
                    comp.l1_batch_number,
                    CAST((comp.updated_at - wit.created_at) AS TIME) AS time_taken,
                    wit.created_at
                FROM
                    proof_compression_jobs_fri AS comp
                    JOIN witness_inputs_fri AS wit ON comp.l1_batch_number = wit.l1_batch_number
                WHERE
                    wit.created_at >  (NOW() - INTERVAL '{days} days')
                ORDER BY
                    time_taken DESC;
                "
            )
        }
        Some(b) => {
            format!(
                "
                SELECT
                    comp.l1_batch_number,
                    CAST((comp.updated_at - wit.created_at) AS TIME) AS time_taken,
                    wit.created_at
                FROM
                    proof_compression_jobs_fri AS comp
                    JOIN witness_inputs_fri AS wit ON comp.l1_batch_number = wit.l1_batch_number
                WHERE
                    comp.l1_batch_number = {}
                ",
                b.0
            )
        }
    };

    let rows = prover_db.fetch_all(query.as_str()).await?;

    let res: eyre::Result<Vec<ProofGenerationTime>> = rows
        .iter()
        .map(proof_generation_time_from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into);

    res
}
