use crate::error::Error as NodeError;
use crate::sol::OpenRankManager::{
    MetaChallengeEvent, MetaComputeRequestEvent, MetaComputeResultEvent, OpenRankManagerInstance,
};
use crate::{
    download_json_metadata_from_s3, download_scores_data_to_file, download_seed_data_to_file,
    download_trust_data_to_file, parse_score_entries_from_file, parse_trust_entries_from_file,
};
use crate::{JobDescription, JobResult};
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::hex::{self, ToHexExt};
use alloy::primitives::Uint;
use alloy::providers::Provider;
use alloy::rpc::types::Log;
use aws_sdk_s3::Client;
use openrank_common::merkle::fixed::DenseMerkleTree;
use openrank_common::merkle::Hash;
use openrank_common::runners::verification_runner::{self, VerificationRunner};
use openrank_common::Domain;
use serde::de::DeserializeOwned;
use sha3::Keccak256;
use std::collections::HashMap;
use std::fs::File;
use std::time::Duration;

use tokio::fs::create_dir_all;
use tracing::{debug, error, info};

pub async fn download_meta<T: DeserializeOwned>(
    client: &Client,
    bucket_name: &str,
    meta_id: String,
) -> Result<T, NodeError> {
    download_json_metadata_from_s3(client, bucket_name, &meta_id).await
}

async fn handle_meta_compute_result<PH: Provider>(
    contract: &OpenRankManagerInstance<(), PH>,
    provider: &PH,
    s3_client: Client,
    bucket_name: String,
    meta_compute_res: MetaComputeResultEvent,
    log: Log,
    meta_compute_request_map: &HashMap<Uint<256, 4>, MetaComputeRequestEvent>,
    meta_challanged_jobs_map: &HashMap<Uint<256, 4>, Log>,
    challenge_window: u64,
) -> Result<(), NodeError> {
    let meta_result: Vec<JobResult> = download_meta(
        &s3_client,
        &bucket_name,
        meta_compute_res.resultsId.encode_hex(),
    )
    .await?;

    info!(
        "ComputeResultEvent: ComputeId({}), Commitment({:#}), ResultsId({:#})",
        meta_compute_res.computeId, meta_compute_res.commitment, meta_compute_res.resultsId
    );
    debug!("Log: {:?}", log);

    let already_challenged = meta_challanged_jobs_map.contains_key(&meta_compute_res.computeId);

    let block = provider
        .get_block(BlockId::Number(BlockNumberOrTag::Latest))
        .await
        .map_err(|e| NodeError::TxError(format!("{e:}")))?
        .ok_or_else(|| NodeError::TxError("Latest block not found".to_string()))?;
    let log_block_number = log
        .block_number
        .ok_or_else(|| NodeError::TxError("Log block number is missing".to_string()))?;
    let log_block = provider
        .get_block(BlockId::Number(BlockNumberOrTag::Number(log_block_number)))
        .await
        .map_err(|e| NodeError::TxError(format!("{e:}")))?
        .ok_or_else(|| NodeError::TxError("Log block not found".to_string()))?;
    if already_challenged {
        return Ok(());
    }

    if !meta_compute_request_map.contains_key(&meta_compute_res.computeId) {
        return Ok(());
    }
    let compute_req = meta_compute_request_map
        .get(&meta_compute_res.computeId)
        .ok_or_else(|| NodeError::TxError("Compute request not found in map".to_string()))?;

    let job_description: Vec<JobDescription> = download_meta(
        &s3_client,
        &bucket_name,
        compute_req.jobDescriptionId.encode_hex(),
    )
    .await?;

    // Create directories for data storage
    create_dir_all("./trust/")
        .await
        .map_err(|e| NodeError::FileError(format!("Failed to create trust directory: {}", e)))?;
    create_dir_all("./seed/")
        .await
        .map_err(|e| NodeError::FileError(format!("Failed to create seed directory: {}", e)))?;
    create_dir_all("./scores/")
        .await
        .map_err(|e| NodeError::FileError(format!("Failed to create scores directory: {}", e)))?;

    // STAGE 1: Download all data files in parallel
    info!("STAGE 1: Downloading all data files in parallel...");

    let download_tasks: Vec<_> = meta_result
        .iter()
        .enumerate()
        .map(|(i, compute_res)| {
            let s3_client = s3_client.clone();
            let bucket_name = bucket_name.clone();
            let trust_id = job_description[i].trust_id.clone();
            let seed_id = job_description[i].seed_id.clone();
            let scores_id = compute_res.scores_id.clone();

            tokio::spawn(async move {
                let trust_file_path = format!("./trust/{}", trust_id);
                let seed_file_path = format!("./seed/{}", seed_id);
                let scores_file_path = format!("./scores/{}", scores_id);

                // Check if trust file already exists
                let (trust_result, trust_downloaded) =
                    if tokio::fs::metadata(&trust_file_path).await.is_ok() {
                        info!("Trust file already exists, skipping download: {}", trust_id);
                        (Ok(()), false)
                    } else {
                        info!(
                            "Downloading trust data for Job {}: TrustId({})",
                            i, trust_id
                        );
                        (
                            download_trust_data_to_file(
                                &s3_client,
                                &bucket_name,
                                &trust_id,
                                &trust_file_path,
                            )
                            .await,
                            true,
                        )
                    };

                // Check if seed file already exists
                let (seed_result, seed_downloaded) =
                    if tokio::fs::metadata(&seed_file_path).await.is_ok() {
                        info!("Seed file already exists, skipping download: {}", seed_id);
                        (Ok(()), false)
                    } else {
                        info!("Downloading seed data for Job {}: SeedId({})", i, seed_id);
                        (
                            download_seed_data_to_file(
                                &s3_client,
                                &bucket_name,
                                &seed_id,
                                &seed_file_path,
                            )
                            .await,
                            true,
                        )
                    };

                // Check if scores file already exists
                let (scores_result, scores_downloaded) =
                    if tokio::fs::metadata(&scores_file_path).await.is_ok() {
                        info!(
                            "Scores file already exists, skipping download: {}",
                            scores_id
                        );
                        (Ok(()), false)
                    } else {
                        info!(
                            "Downloading scores data for Job {}: ScoresId({})",
                            i, scores_id
                        );
                        (
                            download_scores_data_to_file(
                                &s3_client,
                                &bucket_name,
                                &scores_id,
                                &scores_file_path,
                            )
                            .await,
                            true,
                        )
                    };

                // Return results with download status
                (
                    trust_result,
                    seed_result,
                    scores_result,
                    trust_downloaded,
                    seed_downloaded,
                    scores_downloaded,
                    i,
                    trust_id,
                    seed_id,
                    scores_id,
                )
            })
        })
        .collect();

    // Wait for all downloads to complete
    let download_results = futures_util::future::join_all(download_tasks).await;

    // Check for errors and count downloads vs skips
    let mut trust_downloads = 0;
    let mut seed_downloads = 0;
    let mut scores_downloads = 0;

    for result in download_results {
        let (
            trust_result,
            seed_result,
            scores_result,
            trust_downloaded,
            seed_downloaded,
            scores_downloaded,
            _i,
            trust_id,
            seed_id,
            scores_id,
        ) = result.map_err(|e| NodeError::TxError(format!("Download task failed: {}", e)))?;

        trust_result.map_err(|e| {
            NodeError::FileError(format!(
                "Failed to download trust data for {}: {}",
                trust_id, e
            ))
        })?;
        seed_result.map_err(|e| {
            NodeError::FileError(format!(
                "Failed to download seed data for {}: {}",
                seed_id, e
            ))
        })?;
        scores_result.map_err(|e| {
            NodeError::FileError(format!(
                "Failed to download scores data for {}: {}",
                scores_id, e
            ))
        })?;

        if trust_downloaded {
            trust_downloads += 1;
        }
        if seed_downloaded {
            seed_downloads += 1;
        }
        if scores_downloaded {
            scores_downloads += 1;
        }
    }

    let trust_skips = meta_result.len() - trust_downloads;
    let seed_skips = meta_result.len() - seed_downloads;
    let scores_skips = meta_result.len() - scores_downloads;

    info!(
        "STAGE 1 complete: Trust files (downloaded: {}, skipped: {}), Seed files (downloaded: {}, skipped: {}), Scores files (downloaded: {}, skipped: {})",
        trust_downloads, trust_skips, seed_downloads, seed_skips, scores_downloads, scores_skips
    );

    // STAGE 2: Verification compute in parallel
    info!("STAGE 2: Running verification compute...");

    let mut global_result = true;
    let mut sub_job_failed = 0;

    let commitments: Vec<String> = meta_result
        .iter()
        .map(|res| res.commitment.clone())
        .collect();
    for (i, compute_res) in meta_result.iter().enumerate() {
        let trust_id = job_description[i].trust_id.clone();
        let seed_id = job_description[i].seed_id.clone();
        let scores_id = compute_res.scores_id.clone();
        let commitment = compute_res.commitment.clone();

        info!(
            "Running verification for Job {}: TrustId({}), SeedId({}), ScoresId({})",
            i, trust_id, seed_id, scores_id
        );

        let trust_file = File::open(&format!("./trust/{}", trust_id))
            .map_err(|e| NodeError::FileError(format!("Failed to open trust file: {e:}")))?;
        let seed_file = File::open(&format!("./seed/{}", seed_id))
            .map_err(|e| NodeError::FileError(format!("Failed to open seed file: {e:}")))?;
        let scores_file = File::open(&format!("./scores/{}", scores_id))
            .map_err(|e| NodeError::FileError(format!("Failed to open scores file: {e:}")))?;

        let trust_entries = parse_trust_entries_from_file(trust_file)?;
        let seed_entries = parse_score_entries_from_file(seed_file)?;
        let scores_entries = parse_score_entries_from_file(scores_file)?;

        let mock_domain = Domain::default();
        let mut runner = VerificationRunner::new(&[mock_domain.clone()]);
        runner
            .update_trust_map(mock_domain.clone(), trust_entries.to_vec())
            .map_err(NodeError::VerificationRunnerError)?;
        runner
            .update_seed_map(mock_domain.clone(), seed_entries.to_vec())
            .map_err(NodeError::VerificationRunnerError)?;
        runner.update_commitment(
            Hash::from_slice(i.to_be_bytes().as_slice()),
            Hash::from_slice(
                hex::decode(commitment.clone())
                    .map_err(|e| NodeError::HexError(e))?
                    .as_slice(),
            ),
        );
        runner
            .update_scores(
                mock_domain.clone(),
                Hash::from_slice(i.to_be_bytes().as_slice()),
                scores_entries,
            )
            .map_err(NodeError::VerificationRunnerError)?;
        let result = runner
            .verify_job(mock_domain, Hash::from_slice(i.to_be_bytes().as_slice()))
            .map_err(NodeError::VerificationRunnerError)?;

        info!("Verification completed for Job {}: Result({})", i, result);

        if !result {
            global_result = false;
            sub_job_failed = i;
            break;
        }
    }

    info!("STAGE 2 complete: Verification compute done.");

    let commitment_tree = DenseMerkleTree::<Keccak256>::new(
        commitments
            .iter()
            .map(|x| {
                let decoded = hex::decode(x).map_err(|e| NodeError::HexError(e))?;
                Ok(Hash::from_slice(decoded.as_slice()))
            })
            .collect::<Result<Vec<_>, NodeError>>()?
            .into_iter()
            .collect(),
    )
    .map_err(|e| NodeError::VerificationRunnerError(verification_runner::Error::Merkle(e)))?;
    let meta_commitment = commitment_tree
        .root()
        .map_err(|e| NodeError::VerificationRunnerError(verification_runner::Error::Merkle(e)))?;
    let commitment_result = meta_commitment.to_hex() == meta_compute_res.commitment.encode_hex();
    if !commitment_result {
        global_result = false;
    }

    info!("Global result: Result({})", global_result);

    let challenge_window_open =
        (block.header.timestamp - log_block.header.timestamp) < challenge_window;
    info!("Challenge window open: {}", challenge_window_open);

    if !global_result {
        info!("Submitting challenge. Calling 'metaSubmitChallenge'");
        let res = contract
            .submitMetaChallenge(meta_compute_res.computeId, sub_job_failed as u32)
            .send()
            .await;
        if let Ok(res) = res {
            match res.watch().await {
                Ok(tx_res) => {
                    info!("'metaSubmitChallenge' completed. Tx Hash({:#})", tx_res);
                }
                Err(e) => {
                    error!("Failed to watch transaction: {}", e);
                }
            }
        } else {
            let err = res.unwrap_err();
            error!("'metaSubmitChallenge' failed. {}", err);
        }
    }

    Ok(())
}

pub async fn run<P: Provider>(
    manager_contract: OpenRankManagerInstance<(), P>,
    provider: P,
    s3_client: Client,
    bucket_name: &str,
    block_history: u64,
    log_pull_seconds: u64,
) -> Result<(), NodeError> {
    let challenge_window = manager_contract.CHALLENGE_WINDOW().call().await.unwrap();
    let current_block = provider.get_block_number().await.unwrap();
    let starting_block = current_block - block_history;
    let mut meta_compute_request_map = HashMap::new();
    let mut meta_challanged_jobs_map = HashMap::new();
    // Meta jobs events
    let meta_compute_result_filter = manager_contract
        .MetaComputeResultEvent_filter()
        .from_block(BlockNumberOrTag::Number(starting_block))
        .to_block(BlockNumberOrTag::Latest)
        .filter;
    let meta_compute_request_filter = manager_contract
        .MetaComputeRequestEvent_filter()
        .from_block(BlockNumberOrTag::Number(starting_block))
        .to_block(BlockNumberOrTag::Latest)
        .filter;
    let meta_compute_challenge_filter = manager_contract
        .MetaChallengeEvent_filter()
        .from_block(BlockNumberOrTag::Number(starting_block))
        .to_block(BlockNumberOrTag::Latest)
        .filter;

    info!("Pulling historical logs (last {} blocks)...", block_history);

    let result_logs = provider
        .get_logs(&meta_compute_result_filter)
        .await
        .unwrap();
    let request_logs = provider
        .get_logs(&meta_compute_request_filter)
        .await
        .unwrap();
    let challenge_logs = provider
        .get_logs(&meta_compute_challenge_filter)
        .await
        .unwrap();

    for log in request_logs {
        let res: Log<MetaComputeRequestEvent> = log.log_decode().unwrap();
        let compute_req = res.data();
        meta_compute_request_map.insert(compute_req.computeId, compute_req.clone());
    }

    for log in challenge_logs {
        let res: Log<MetaChallengeEvent> = log.log_decode().unwrap();
        let challenge = res.data();
        meta_challanged_jobs_map.insert(challenge.computeId, log);
    }

    for log in result_logs {
        let res: Log<MetaComputeResultEvent> = log.log_decode().unwrap();
        let meta_compute_res = res.data();
        if let Err(e) = handle_meta_compute_result(
            &manager_contract,
            &provider,
            s3_client.clone(),
            bucket_name.to_string(),
            meta_compute_res.clone(),
            log,
            &meta_compute_request_map,
            &meta_challanged_jobs_map,
            challenge_window._0,
        )
        .await
        {
            error!("Error handling meta compute result: {}", e);
        }
    }

    info!("Pulling new events...");

    let mut interval = tokio::time::interval(Duration::from_secs(log_pull_seconds));
    let mut latest_processed_block = current_block;

    loop {
        interval.tick().await; // Wait for the next tick

        let current_block = provider.get_block_number().await.unwrap();

        let meta_compute_result_filter = manager_contract
            .MetaComputeResultEvent_filter()
            .from_block(BlockNumberOrTag::Number(latest_processed_block))
            .to_block(BlockNumberOrTag::Number(current_block))
            .filter;
        let meta_compute_request_filter = manager_contract
            .MetaComputeRequestEvent_filter()
            .from_block(BlockNumberOrTag::Number(latest_processed_block))
            .to_block(BlockNumberOrTag::Number(current_block))
            .filter;
        let meta_compute_challenge_filter = manager_contract
            .MetaChallengeEvent_filter()
            .from_block(BlockNumberOrTag::Number(latest_processed_block))
            .to_block(BlockNumberOrTag::Number(current_block))
            .filter;

        let result_logs = provider
            .get_logs(&meta_compute_result_filter)
            .await
            .unwrap();
        let request_logs = provider
            .get_logs(&meta_compute_request_filter)
            .await
            .unwrap();
        let challenge_logs = provider
            .get_logs(&meta_compute_challenge_filter)
            .await
            .unwrap();

        for log in request_logs {
            let res: Log<MetaComputeRequestEvent> = log.log_decode().unwrap();
            let compute_req = res.data();
            meta_compute_request_map.insert(compute_req.computeId, compute_req.clone());
        }

        for log in challenge_logs {
            let res: Log<MetaChallengeEvent> = log.log_decode().unwrap();
            let challenge = res.data();
            meta_challanged_jobs_map.insert(challenge.computeId, log);
        }

        for log in result_logs {
            let res: Log<MetaComputeResultEvent> = log.log_decode().unwrap();
            let meta_compute_res = res.data();
            if let Err(e) = handle_meta_compute_result(
                &manager_contract,
                &provider,
                s3_client.clone(),
                bucket_name.to_string(),
                meta_compute_res.clone(),
                log,
                &meta_compute_request_map,
                &meta_challanged_jobs_map,
                challenge_window._0,
            )
            .await
            {
                error!("Error handling meta compute result: {}", e);
            }
        }

        latest_processed_block = current_block;
    }
}
