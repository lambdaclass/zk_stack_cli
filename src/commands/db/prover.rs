use crate::{
    config::ZKSyncConfig,
    utils::{
        config::confirm,
        db::{
            prover::{
                find_map_stuck_wg_jobs_in_aggregation_round,
                find_stuck_prover_jobs_in_aggregation_round, map_bwg_info, map_leaf_wg_info,
                map_node_wg_info, map_recursion_tip_wg_info, map_scheduler_wg_info,
            },
            queries::{get_compressor_job_status, restart_batch_proof},
        },
        messages::{
            DATABASE_PROVER_RESTART_ALREADY_PROVED_BATCH_PROOF_CONFIRMATION_MSG,
            DATABASE_PROVER_RESTART_BATCH_PROOF_CONFIRMATION_MSG,
        },
    },
};
use clap::Subcommand;
use eyre::ContextCompat;
use zksync_ethers_rs::types::zksync::basic_fri_types::AggregationRound;

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "List all the stuck batch proofs.", visible_alias = "stuck")]
    StuckBatchProofs,
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let mut prover_db = cfg
            .db
            .context("DB config missing")?
            .prover
            .acquire()
            .await?;
        match self {
            Command::StuckBatchProofs => {
                let mut aggregation_round = AggregationRound::BasicCircuits;
                find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_bwg_info,
                    &mut prover_db,
                )
                .await?;
                find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                    .await?;

                aggregation_round = AggregationRound::LeafAggregation;
                find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_leaf_wg_info,
                    &mut prover_db,
                )
                .await?;
                find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                    .await?;

                aggregation_round = AggregationRound::NodeAggregation;
                find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_node_wg_info,
                    &mut prover_db,
                )
                .await?;
                find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                    .await?;

                aggregation_round = AggregationRound::RecursionTip;
                find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_recursion_tip_wg_info,
                    &mut prover_db,
                )
                .await?;
                find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                    .await?;

                aggregation_round = AggregationRound::Scheduler;
                find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_scheduler_wg_info,
                    &mut prover_db,
                )
                .await?;
                find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                    .await?;
            }
        };

        Ok(())
    }
}
