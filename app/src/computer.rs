use crate::error::Error as NodeError;
use crate::sol::OpenRankManager::{
    MetaComputeRequestEvent, MetaComputeResultEvent, OpenRankManagerInstance,
};
use alloy::eips::BlockNumberOrTag;
use alloy::hex::{self, ToHexExt};
use alloy::primitives::FixedBytes;
use alloy::providers::Provider;
use alloy::rpc::types::Log;
use aws_sdk_s3::Client;
use openrank_common::{JobDescription, JobResult};

use crate::{
    create_csv_and_hash_from_scores, download_meta, download_seed_data_to_file,
    download_trust_data_to_file, parse_score_entries_from_file, parse_trust_entries_from_file,
    upload_file_to_s3_streaming, upload_meta,
};
use openrank_common::merkle::fixed::DenseMerkleTree;
use openrank_common::merkle::Hash;
use openrank_common::runner::{self, ComputeRunner};

use sha3::Keccak256;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use std::time::{Duration, Instant};
use tokio::fs::create_dir_all;
use tracing::{debug, error, info};

async fn handle_meta_compute_request<PH: Provider>(
    contract: &OpenRankManagerInstance<PH>,
    s3_client: Client,
    bucket_name: String,
    meta_compute_req: MetaComputeRequestEvent,
    log: Log,
) -> Result<(), NodeError> {
    let start = Instant::now();
    let meta_job: Vec<JobDescription> = download_meta(
        &s3_client,
        &bucket_name,
        meta_compute_req.jobDescriptionId.encode_hex(),
    )
    .await?;
    info!(
        "MetaComputeRequestEvent: ComputeId({})",
        meta_compute_req.computeId.to_string()
    );
    debug!("Log: {:?}", log);

    // Create directories for data storage
    create_dir_all(&format!("./trust/"))
        .await
        .map_err(|e| NodeError::FileError(format!("Failed to create trust directory: {}", e)))?;
    create_dir_all(&format!("./seed/"))
        .await
        .map_err(|e| NodeError::FileError(format!("Failed to create seed directory: {}", e)))?;
    create_dir_all("./scores/")
        .await
        .map_err(|e| NodeError::FileError(format!("Failed to create scores directory: {}", e)))?;

    // STAGE 1: Download all data files in parallel
    info!("STAGE 1: Downloading all data files in parallel...");

    let download_tasks: Vec<_> = meta_job
        .iter()
        .map(|compute_req| {
            let s3_client = s3_client.clone();
            let bucket_name = bucket_name.clone();
            let trust_id = compute_req.trust_id.clone();
            let seed_id = compute_req.seed_id.clone();
            let trust_id_bytes =
                FixedBytes::<32>::from_slice(hex::decode(trust_id.clone()).unwrap().as_slice());
            let seed_id_bytes =
                FixedBytes::<32>::from_slice(hex::decode(seed_id.clone()).unwrap().as_slice());

            tokio::spawn(async move {
                let trust_file_path = format!("./trust/{}", trust_id);
                let seed_file_path = format!("./seed/{}", seed_id);

                // Check if trust file already exists
                let (trust_result, trust_downloaded) =
                    if tokio::fs::metadata(&trust_file_path).await.is_ok() {
                        info!(
                            "Trust file already exists, skipping download: TrustId({:#})",
                            trust_id_bytes
                        );
                        (Ok(()), false)
                    } else {
                        info!("Downloading data: TrustId({:#})", trust_id_bytes);
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
                        info!("Skipping download: SeedId({:#})", seed_id_bytes);
                        (Ok(()), false)
                    } else {
                        info!("Downloading data: SeedId({:#})", seed_id);
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

                // Return results with download status
                (
                    trust_result,
                    seed_result,
                    trust_downloaded,
                    seed_downloaded,
                    trust_id,
                    seed_id,
                )
            })
        })
        .collect();

    // Wait for all downloads to complete
    let download_results = futures_util::future::join_all(download_tasks).await;

    // Check for errors and count downloads vs skips
    let mut trust_downloads = 0;
    let mut seed_downloads = 0;

    for result in download_results {
        let (trust_result, seed_result, trust_downloaded, seed_downloaded, trust_id, seed_id) =
            result.map_err(|e| NodeError::TxError(format!("Download task failed: {}", e)))?;

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

        if trust_downloaded {
            trust_downloads += 1;
        }
        if seed_downloaded {
            seed_downloads += 1;
        }
    }

    let trust_skips = meta_job.len() - trust_downloads;
    let seed_skips = meta_job.len() - seed_downloads;

    info!(
        "STAGE 1 complete: Trust files (downloaded: {}, skipped: {}), Seed files (downloaded: {}, skipped: {})",
        trust_downloads, trust_skips, seed_downloads, seed_skips
    );

    // STAGE 2: Compute scores and save to CSV files in parallel
    info!("STAGE 2: Computing scores and saving to CSV files in parallel...");

    let mut job_results = Vec::new();
    let mut commitments = Vec::new();
    for compute_req in meta_job {
        let trust_id = compute_req.trust_id.clone();
        let seed_id = compute_req.seed_id.clone();
        let trust_id_bytes =
            FixedBytes::<32>::from_slice(hex::decode(trust_id.clone()).unwrap().as_slice());
        let seed_id_bytes =
            FixedBytes::<32>::from_slice(hex::decode(seed_id.clone()).unwrap().as_slice());

        info!(
            "Computing scores for SubJob: TrustId({:#}), SeedId({:#})",
            trust_id_bytes, seed_id_bytes
        );

        let trust_file = File::open(&format!("./trust/{}", trust_id))
            .map_err(|e| NodeError::FileError(format!("Failed to open trust file: {e:}")))?;
        let seed_file = File::open(&format!("./seed/{}", seed_id))
            .map_err(|e| NodeError::FileError(format!("Failed to open seed file: {e:}")))?;

        let trust_entries = parse_trust_entries_from_file(trust_file)?;
        let seed_entries = parse_score_entries_from_file(seed_file)?;

        // Core compute operations
        let mut runner = ComputeRunner::new();
        runner
            .update_trust_map(trust_entries.to_vec())
            .map_err(NodeError::ComputeRunnerError)?;
        runner
            .update_seed_map(seed_entries.to_vec())
            .map_err(NodeError::ComputeRunnerError)?;
        runner
            .compute_et(compute_req.alpha, compute_req.delta)
            .map_err(NodeError::ComputeRunnerError)?;
        let scores = runner
            .get_compute_scores()
            .map_err(NodeError::ComputeRunnerError)?;
        runner
            .create_compute_tree()
            .map_err(NodeError::ComputeRunnerError)?;
        let compute_root = runner
            .get_root_hash()
            .map_err(NodeError::ComputeRunnerError)?;

        // Create CSV file and compute hash
        let (file_bytes, scores_id) = create_csv_and_hash_from_scores(scores)?;

        // Save CSV to local file
        let scores_file_path = format!("./scores/{}.csv", hex::encode(&scores_id));
        let mut scores_file = File::create(&scores_file_path)
            .map_err(|e| NodeError::FileError(format!("Failed to create scores file: {}", e)))?;
        scores_file
            .write_all(&file_bytes)
            .map_err(|e| NodeError::FileError(format!("Failed to write scores file: {}", e)))?;

        let commitment_bytes = FixedBytes::<32>::from_slice(compute_root.inner());
        let scores_id_bytes = FixedBytes::<32>::from_slice(scores_id.as_slice());
        let commitment = hex::encode(compute_root.inner());
        let scores_id_hex = hex::encode(scores_id.clone());
        let job_result = JobResult::new(scores_id_hex.clone(), commitment);

        info!(
            "Core compute completed: ScoresId({:#}), Commitment({:#})",
            scores_id_bytes, commitment_bytes
        );

        job_results.push(job_result);
        commitments.push(Hash::from_slice(commitment_bytes.as_slice()));
    }

    info!("STAGE 2 complete: All scores computed and saved to CSV files in parallel");

    // STAGE 3: Upload all scores files to S3 in parallel
    info!("STAGE 3: Uploading all scores files to S3 in parallel...");

    let upload_tasks: Vec<_> = job_results
        .iter()
        .map(|job_result| {
            let s3_client = s3_client.clone();
            let bucket_name = bucket_name.clone();
            let scores_id = job_result.scores_id.clone();
            let scores_id_bytes =
                FixedBytes::<32>::from_slice(hex::decode(scores_id.clone()).unwrap().as_slice());

            tokio::spawn(async move {
                info!("Uploading scores data for ScoresId({:#})", scores_id_bytes);

                let scores_file_path = format!("./scores/{}.csv", scores_id);
                let upload_result = upload_file_to_s3_streaming(
                    &s3_client,
                    &bucket_name,
                    &format!("scores/{}", scores_id),
                    &scores_file_path,
                )
                .await
                .map_err(|e| NodeError::FileError(format!("Failed to upload scores file: {}", e)));

                if upload_result.is_ok() {
                    info!("Upload complete for ScoresId({:#})", scores_id_bytes);
                }

                upload_result.map(|_| scores_id.clone())
            })
        })
        .collect();

    // Wait for all uploads to complete
    let upload_results = futures_util::future::join_all(upload_tasks).await;

    // Check for errors
    for result in upload_results {
        let upload_result =
            result.map_err(|e| NodeError::TxError(format!("Upload task failed: {}", e)))?;
        upload_result
            .map_err(|e| NodeError::FileError(format!("Failed to upload scores file: {}", e)))?;
    }

    info!("STAGE 3 complete: All scores files uploaded to S3 in parallel");

    let commitment_tree = DenseMerkleTree::<Keccak256>::new(commitments)
        .map_err(|e| NodeError::ComputeRunnerError(runner::Error::Merkle(e)))?;
    let meta_commitment = commitment_tree
        .root()
        .map_err(|e| NodeError::ComputeRunnerError(runner::Error::Merkle(e)))?;

    let meta_id = upload_meta(&s3_client, &bucket_name, job_results).await?;

    let meta_commitment_bytes = FixedBytes::from_slice(meta_commitment.inner());
    let meta_id_bytes = FixedBytes::from_slice(
        hex::decode(meta_id)
            .map_err(|e| NodeError::HexError(e))?
            .as_slice(),
    );

    info!("Posting commitment on-chain. Calling: 'submitMetaComputeResult'");
    let res = contract
        .submitMetaComputeResult(
            meta_compute_req.computeId,
            meta_commitment_bytes,
            meta_id_bytes,
        )
        .send()
        .await
        .map_err(|e| NodeError::TxError(format!("{e:}")))?;
    let tx_hash = *res.tx_hash();
    info!(
        "'submitMetaComputeResult' submitted: Tx Hash({:#})",
        tx_hash
    );

    let elapsed = start.elapsed();
    info!("Total compute time: {:?}", elapsed);

    Ok(())
}

pub async fn run<PH: Provider>(
    contract: OpenRankManagerInstance<PH>,
    provider: PH,
    s3_client: Client,
    bucket_name: &str,
    block_history: u64,
    log_pull_seconds: u64,
) -> Result<(), NodeError> {
    let current_block = provider
        .get_block_number()
        .await
        .map_err(|e| NodeError::TxError(format!("Failed to get block number: {}", e)))?;
    let starting_block = current_block - block_history;
    // Meta jobs events
    let meta_compute_result_filter = contract
        .MetaComputeResultEvent_filter()
        .from_block(BlockNumberOrTag::Number(starting_block))
        .to_block(BlockNumberOrTag::Latest)
        .filter;
    let meta_compute_request_filter = contract
        .MetaComputeRequestEvent_filter()
        .from_block(BlockNumberOrTag::Number(starting_block))
        .to_block(BlockNumberOrTag::Latest)
        .filter;

    info!("Pulling historical logs (last {} blocks)...", block_history);

    let result_logs = provider
        .get_logs(&meta_compute_result_filter)
        .await
        .map_err(|e| NodeError::TxError(format!("Failed to get result logs: {}", e)))?;
    let request_logs = provider
        .get_logs(&meta_compute_request_filter)
        .await
        .map_err(|e| NodeError::TxError(format!("Failed to get request logs: {}", e)))?;

    let mut finished_jobs = HashSet::new();
    for log in result_logs {
        let res: Log<MetaComputeResultEvent> = log
            .log_decode()
            .map_err(|e| NodeError::TxError(format!("Failed to decode result log: {}", e)))?;
        finished_jobs.insert(res.data().computeId);
    }

    for log in request_logs {
        let res: Log<MetaComputeRequestEvent> = log
            .log_decode()
            .map_err(|e| NodeError::TxError(format!("Failed to decode request log: {}", e)))?;
        if finished_jobs.contains(&res.data().computeId) {
            continue;
        }
        if let Err(e) = handle_meta_compute_request(
            &contract,
            s3_client.clone(),
            bucket_name.to_string(),
            res.data().clone(),
            log,
        )
        .await
        {
            error!("Error handling meta compute request: {}", e);
        }
    }

    info!("Pulling new events...");

    let mut interval = tokio::time::interval(Duration::from_secs(log_pull_seconds));
    let mut latest_processed_block = current_block;

    loop {
        interval.tick().await; // Wait for the next tick

        let current_block = match provider.get_block_number().await {
            Ok(block) => block,
            Err(e) => {
                error!("Error getting current block number: {}", e);
                continue;
            }
        };

        let meta_compute_result_filter = contract
            .MetaComputeResultEvent_filter()
            .from_block(BlockNumberOrTag::Number(latest_processed_block))
            .to_block(BlockNumberOrTag::Number(current_block))
            .filter;
        let meta_compute_request_filter = contract
            .MetaComputeRequestEvent_filter()
            .from_block(BlockNumberOrTag::Number(latest_processed_block))
            .to_block(BlockNumberOrTag::Number(current_block))
            .filter;

        let result_logs = match provider.get_logs(&meta_compute_result_filter).await {
            Ok(logs) => logs,
            Err(e) => {
                error!("Error getting result logs: {}", e);
                continue;
            }
        };
        let request_logs = match provider.get_logs(&meta_compute_request_filter).await {
            Ok(logs) => logs,
            Err(e) => {
                error!("Error getting request logs: {}", e);
                continue;
            }
        };

        for log in result_logs {
            let res: Log<MetaComputeResultEvent> = match log.log_decode() {
                Ok(decoded) => decoded,
                Err(e) => {
                    error!("Error decoding result log: {}", e);
                    continue;
                }
            };
            finished_jobs.insert(res.data().computeId);
        }

        for log in request_logs {
            let res: Log<MetaComputeRequestEvent> = match log.log_decode() {
                Ok(decoded) => decoded,
                Err(e) => {
                    error!("Error decoding request log: {}", e);
                    continue;
                }
            };
            if finished_jobs.contains(&res.data().computeId) {
                continue;
            }
            if let Err(e) = handle_meta_compute_request(
                &contract,
                s3_client.clone(),
                bucket_name.to_string(),
                res.data().clone(),
                log,
            )
            .await
            {
                error!("Error handling meta compute request: {}", e);
            }
        }

        latest_processed_block = current_block;
    }
}
