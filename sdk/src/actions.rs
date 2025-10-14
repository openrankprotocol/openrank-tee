use crate::BUCKET_NAME;
use alloy::hex::{self};
use aws_sdk_s3::{primitives::ByteStream, Client, Error as AwsError};
use openrank_common::{
    merkle::Hash,
    runners::{
        compute_runner::{self, ComputeRunner},
        verification_runner::{self, VerificationRunner},
    },
    tx::trust::{ScoreEntry, TrustEntry},
    Domain,
};
use serde::{de::DeserializeOwned, Serialize};
use sha3::{Digest, Keccak256};
use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};
use tracing::{debug, info};

/// Helper function to validate trust CSV format
fn validate_trust_csv(path: &str) -> Result<(), csv::Error> {
    let file = File::open(path).unwrap();
    let mut reader = csv::Reader::from_reader(file);
    for result in reader.records() {
        let record: csv::StringRecord = result?;
        let (_, _, _): (String, String, f32) = record.deserialize(None)?;
    }
    Ok(())
}

/// Helper function to validate score CSV format
fn validate_score_csv(path: &str) -> Result<(), csv::Error> {
    let file = File::open(path).unwrap();
    let mut reader = csv::Reader::from_reader(file);
    for result in reader.records() {
        let record: csv::StringRecord = result?;
        let (_, _): (String, f32) = record.deserialize(None)?;
    }
    Ok(())
}

pub async fn upload_trust(client: Client, path: String) -> Result<String, AwsError> {
    let mut f = File::open(path.clone()).unwrap();
    let mut file_bytes = Vec::new();
    f.read_to_end(&mut file_bytes).unwrap();
    let body = ByteStream::from(file_bytes.clone());

    let mut hasher = Keccak256::new();
    hasher.write_all(&mut file_bytes).unwrap();
    let hash = hasher.finalize().to_vec();

    validate_trust_csv(&path).unwrap();

    info!("Uploading trust data: {}", hex::encode(hash.clone()));

    client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(format!("trust/{}", hex::encode(hash.clone())))
        .body(body)
        .send()
        .await?;

    Ok(hex::encode(hash))
}

pub async fn upload_seed(client: Client, path: String) -> Result<String, AwsError> {
    let mut f = File::open(path.clone()).unwrap();
    let mut file_bytes = Vec::new();
    f.read_to_end(&mut file_bytes).unwrap();
    let body = ByteStream::from(file_bytes.clone());

    let mut hasher = Keccak256::new();
    hasher.write_all(&mut file_bytes).unwrap();
    let hash = hasher.finalize().to_vec();

    validate_score_csv(&path).unwrap();

    info!("Uploading seed data: {}", hex::encode(hash.clone()));

    client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(format!("seed/{}", hex::encode(hash.clone())))
        .body(body)
        .send()
        .await?;

    Ok(hex::encode(hash))
}

pub async fn _download_trust(
    client: Client,
    trust_id: String,
    path: String,
) -> Result<(), AwsError> {
    let mut file = File::create(path).unwrap();
    let mut res = client
        .get_object()
        .bucket(BUCKET_NAME)
        .key(format!("trust/{}", trust_id))
        .send()
        .await?;
    while let Some(bytes) = res.body.next().await {
        file.write(&bytes.unwrap()).unwrap();
    }
    Ok(())
}

pub async fn _download_seed(client: Client, seed_id: String, path: String) -> Result<(), AwsError> {
    let mut file = File::create(path).unwrap();
    let mut res = client
        .get_object()
        .bucket(BUCKET_NAME)
        .key(format!("seed/{}", seed_id))
        .send()
        .await?;
    while let Some(bytes) = res.body.next().await {
        file.write(&bytes.unwrap()).unwrap();
    }
    Ok(())
}

pub async fn download_scores(
    client: Client,
    scores_id: String,
    path: String,
) -> Result<(), AwsError> {
    // Download the scores data from S3
    let mut res = client
        .get_object()
        .bucket(BUCKET_NAME)
        .key(format!("scores/{}", scores_id))
        .send()
        .await?;
    debug!("{:?}", res);

    // Collect all bytes into a vector
    let mut csv_bytes = Vec::new();
    while let Some(bytes) = res.body.next().await {
        csv_bytes.extend_from_slice(&bytes.unwrap());
    }

    // Parse CSV bytes into ScoreEntry objects
    let mut scores = parse_csv_to_scores(&csv_bytes).expect("Failed to parse CSV data");

    // Sort scores from highest to lowest value
    scores.sort_by(|a, b| {
        b.value()
            .partial_cmp(a.value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Write sorted scores to CSV file
    write_scores_to_csv(&scores, &path).expect("Failed to write CSV file");

    Ok(())
}

/// Parse CSV bytes into a vector of ScoreEntry objects
fn parse_csv_to_scores(csv_bytes: &[u8]) -> Result<Vec<ScoreEntry>, csv::Error> {
    let mut reader = csv::Reader::from_reader(csv_bytes);
    let mut scores = Vec::new();

    for result in reader.records() {
        let record = result?;
        let id: String = record.get(0).unwrap_or("").to_string();
        let value: f32 = record.get(1).unwrap_or("0.0").parse().unwrap_or(0.0);
        scores.push(ScoreEntry::new(id, value));
    }

    Ok(scores)
}

/// Write ScoreEntry objects to CSV file with i,v headers
fn write_scores_to_csv(
    scores: &[ScoreEntry],
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(file_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    // Write header
    wtr.write_record(&["i", "v"])?;

    // Write scores
    for score in scores {
        wtr.write_record(&[score.id(), &score.value().to_string()])?;
    }

    wtr.flush()?;
    Ok(())
}

pub async fn upload_meta<T: Serialize>(client: Client, meta: T) -> Result<String, AwsError> {
    let mut bytes = serde_json::to_vec(&meta).unwrap();
    let body = ByteStream::from(bytes.clone());

    let mut hasher = Keccak256::new();
    hasher.write_all(&mut bytes).unwrap();
    let hash = hasher.finalize().to_vec();
    client
        .put_object()
        .bucket(BUCKET_NAME)
        .key(format!("meta/{}", hex::encode(hash.clone())))
        .body(body)
        .send()
        .await?;
    Ok(hex::encode(hash))
}

pub async fn download_meta<T: DeserializeOwned>(
    client: Client,
    meta_id: String,
) -> Result<T, AwsError> {
    let res = client
        .get_object()
        .bucket(BUCKET_NAME)
        .key(format!("meta/{}", meta_id))
        .send()
        .await?;
    let res_bytes = res.body.collect().await.unwrap();
    let meta: T = serde_json::from_slice(res_bytes.to_vec().as_slice()).unwrap();
    Ok(meta)
}

pub async fn compute_local(
    trust_entries: &[TrustEntry],
    seed_entries: &[ScoreEntry],
    alpha: Option<f32>,
    delta: Option<f32>,
) -> Result<Vec<ScoreEntry>, compute_runner::Error> {
    let mock_domain = Domain::default();
    let mut runner = ComputeRunner::new(&[mock_domain.clone()]);
    runner.update_trust(mock_domain.clone(), trust_entries.to_vec())?;
    runner.update_seed(mock_domain.clone(), seed_entries.to_vec())?;
    runner.compute(mock_domain.clone(), alpha, delta)?;
    let scores = runner.get_compute_scores(mock_domain.clone())?;
    Ok(scores)
}

pub async fn verify_local(
    trust_entries: &[TrustEntry],
    seed_entries: &[ScoreEntry],
    scores_entries: &[ScoreEntry],
    alpha: Option<f32>,
    delta: Option<f32>,
) -> Result<bool, verification_runner::Error> {
    let mock_domain = Domain::default();
    let mut runner = VerificationRunner::new(&[mock_domain.clone()]);
    runner.update_trust_map(mock_domain.clone(), trust_entries.to_vec())?;
    runner.update_seed_map(mock_domain.clone(), seed_entries.to_vec())?;
    runner.update_scores(
        mock_domain.clone(),
        Hash::default(),
        scores_entries.to_vec(),
    )?;
    let result = runner.verify_scores(mock_domain, Hash::default(), alpha, delta)?;
    Ok(result)
}

pub fn save_json_to_file<T: Serialize>(data: T, file: &Path) -> Result<(), std::io::Error> {
    let file = File::create(file.to_path_buf())?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &data)?;
    writer.flush()?;
    Ok(())
}
