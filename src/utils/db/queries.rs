use crate::utils::db::{types::ProverJobFriInfo, CURRENT_MAX_ATTEMPTS};
use sqlx::{pool::PoolConnection, postgres::PgRow, Executor, FromRow, Postgres};
use zksync_ethers_rs::types::zksync::basic_fri_types::AggregationRound;

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

fn input_table_name_for(aggregation_round: AggregationRound) -> &'static str {
    match aggregation_round {
        AggregationRound::BasicCircuits => "witness_inputs_fri",
        AggregationRound::LeafAggregation => "leaf_aggregation_witness_jobs_fri",
        AggregationRound::NodeAggregation => "node_aggregation_witness_jobs_fri",
        AggregationRound::RecursionTip => "recursion_tip_witness_jobs_fri",
        AggregationRound::Scheduler => "scheduler_witness_jobs_fri",
    }
}
