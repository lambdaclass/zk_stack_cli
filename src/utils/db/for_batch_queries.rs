use crate::utils::db::types::ProverJobFriInfo;
use eyre::Context;
use sqlx::{pool::PoolConnection, Executor, Postgres, Row};
use std::str::FromStr;
use zksync_ethers_rs::types::zksync::prover_dal::{
    BasicWitnessGeneratorJobInfo, LeafWitnessGeneratorJobInfo, NodeWitnessGeneratorJobInfo,
    ProofCompressionJobInfo, SchedulerWitnessGeneratorJobInfo,
};
use zksync_ethers_rs::types::zksync::{
    basic_fri_types::AggregationRound, prover_dal::WitnessJobStatus, L1BatchNumber,
};

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
) -> eyre::Result<Option<Vec<LeafWitnessGeneratorJobInfo>>> {
    let query = format!(
        "
        SELECT *
        FROM leaf_aggregation_witness_jobs_fri
        WHERE
            l1_batch_number = {}
        ",
        l1_batch_number.0,
    );

    let rows = prover_db.fetch_all(query.as_str()).await?;

    let mut answer = Vec::new();

    for row in rows {
        let job_info = LeafWitnessGeneratorJobInfo {
            // is this ok?
            id: row.get::<i32, _>("id").try_into()?,
            l1_batch_number,
            circuit_id: row.get::<i32, _>("circuit_id").try_into()?,
            closed_form_inputs_blob_url: row.get("closed_form_inputs_blob_url"),
            attempts: row.get::<i32, _>("attempts").try_into()?,
            status: {
                let raw_status: String = row.get("status");
                WitnessJobStatus::from_str(&raw_status).context("Parsing WitnessJobStatus")?
            },
            error: row.get("error"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            processing_started_at: row.get("processing_started_at"),
            time_taken: row.get("time_taken"),
            protocol_version: row.get("protocol_version"),
            picked_by: row.get("picked_by"),
            number_of_basic_circuits: row.get("number_of_basic_circuits"),
        };

        answer.push(job_info);
    }

    Ok(Some(answer))
}

async fn get_proof_node_witness_generator_info_for_batch(
    _batch_number: L1BatchNumber,
    _prover_db: &mut PoolConnection<Postgres>,
) -> Vec<NodeWitnessGeneratorJobInfo> {
    todo!()
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
