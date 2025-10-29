pub mod computer;
pub mod error;
pub mod sol;

pub use crate::error::Error;
use aws_sdk_s3::Client as S3Client;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::Write;

/// Creates CSV data from score entries and returns both CSV bytes and its Keccak256 hash.
pub fn create_csv_and_hash_from_scores<I>(scores: I) -> Result<(Vec<u8>, Vec<u8>), Error>
where
    I: IntoIterator<Item = openrank_common::ScoreEntry>,
{
    use sha3::{Digest, Keccak256};

    let scores_vec = Vec::new();
    let mut wtr = csv::Writer::from_writer(scores_vec);
    wtr.write_record(&["i", "v"]).map_err(Error::CsvError)?;

    for score in scores {
        wtr.write_record(&[score.id(), score.value().to_string().as_str()])
            .map_err(Error::CsvError)?;
    }

    let csv_bytes = wtr
        .into_inner()
        .map_err(|e| Error::FileError(format!("Failed to get CSV writer inner data: {}", e)))?;

    let mut hasher = Keccak256::new();
    hasher
        .write_all(&csv_bytes)
        .map_err(|e| Error::FileError(format!("Failed to write to hasher: {}", e)))?;
    let hash = hasher.finalize().to_vec();

    Ok((csv_bytes, hash))
}

/// Creates CSV file from score entries, saves it to disk, and returns its Keccak256 hash.
pub fn create_csv_file_and_hash_from_scores<I>(scores: I, file_path: &str) -> Result<Vec<u8>, Error>
where
    I: IntoIterator<Item = openrank_common::ScoreEntry>,
{
    use sha3::{Digest, Keccak256};
    use std::fs::File;

    let file = File::create(file_path)
        .map_err(|e| Error::FileError(format!("Failed to create file {}: {}", file_path, e)))?;

    let mut wtr = csv::Writer::from_writer(file);
    wtr.write_record(&["i", "v"]).map_err(Error::CsvError)?;

    let mut csv_bytes = Vec::new();
    let mut temp_wtr = csv::Writer::from_writer(&mut csv_bytes);
    temp_wtr
        .write_record(&["i", "v"])
        .map_err(Error::CsvError)?;

    for score in scores {
        let id = score.id();
        let value_str = score.value().to_string();

        // Write to file
        wtr.write_record(&[id, &value_str])
            .map_err(Error::CsvError)?;

        // Write to temp buffer for hashing
        temp_wtr
            .write_record(&[id, &value_str])
            .map_err(Error::CsvError)?;
    }

    // Flush and close file writer
    wtr.flush()
        .map_err(|e| Error::FileError(format!("Failed to flush CSV writer: {}", e)))?;

    // Get bytes for hashing
    let csv_bytes = temp_wtr
        .into_inner()
        .map_err(|e| Error::FileError(format!("Failed to get CSV writer inner data: {}", e)))?;

    let mut hasher = Keccak256::new();
    hasher
        .write_all(&csv_bytes)
        .map_err(|e| Error::FileError(format!("Failed to write to hasher: {}", e)))?;
    let hash = hasher.finalize().to_vec();

    Ok(hash)
}

/// Downloads an S3 object and saves it to a local file.
pub async fn download_s3_object_to_file(
    s3_client: &S3Client,
    bucket_name: &str,
    object_key: &str,
    file_path: &str,
) -> Result<(), Error> {
    let mut file = File::create(file_path)
        .map_err(|e| Error::FileError(format!("Failed to create file {}: {}", file_path, e)))?;

    let mut response = s3_client
        .get_object()
        .bucket(bucket_name)
        .key(object_key)
        .send()
        .await
        .map_err(|e| Error::AwsError(e.into()))?;

    while let Some(bytes) = response.body.next().await {
        let chunk = bytes.map_err(Error::ByteStreamError)?;
        file.write_all(&chunk).map_err(|e| {
            Error::FileError(format!("Failed to write to file {}: {}", file_path, e))
        })?;
    }

    Ok(())
}

/// Downloads an S3 object and returns the data as bytes.
pub async fn download_s3_object_as_bytes(
    s3_client: &S3Client,
    bucket_name: &str,
    object_key: &str,
) -> Result<Vec<u8>, Error> {
    let mut response = s3_client
        .get_object()
        .bucket(bucket_name)
        .key(object_key)
        .send()
        .await
        .map_err(|e| Error::AwsError(e.into()))?;

    let mut data = Vec::new();
    while let Some(bytes) = response.body.next().await {
        let chunk = bytes.map_err(Error::ByteStreamError)?;
        data.extend_from_slice(&chunk);
    }

    Ok(data)
}

/// Uploads raw bytes to S3 with the specified key.
pub async fn upload_bytes_to_s3(
    s3_client: &S3Client,
    bucket_name: &str,
    object_key: &str,
    data: &[u8],
) -> Result<(), Error> {
    use aws_sdk_s3::primitives::ByteStream;

    let body = ByteStream::from(data.to_vec());

    s3_client
        .put_object()
        .bucket(bucket_name)
        .key(object_key)
        .body(body)
        .send()
        .await
        .map_err(|e| Error::AwsError(e.into()))?;

    Ok(())
}

/// Uploads a file to S3 using streaming without loading the entire file into memory.
pub async fn upload_file_to_s3_streaming(
    s3_client: &S3Client,
    bucket_name: &str,
    object_key: &str,
    file_path: &str,
) -> Result<(), Error> {
    use aws_sdk_s3::primitives::ByteStream;
    use tokio::fs::File;

    // Open the file asynchronously
    let file = File::open(file_path)
        .await
        .map_err(|e| Error::FileError(format!("Failed to open file {}: {}", file_path, e)))?;

    // Create a ByteStream from the file
    let body = ByteStream::read_from()
        .file(file)
        .build()
        .await
        .map_err(|e| {
            Error::FileError(format!(
                "Failed to create stream from file {}: {}",
                file_path, e
            ))
        })?;

    // Upload using the streaming body
    s3_client
        .put_object()
        .bucket(bucket_name)
        .key(object_key)
        .body(body)
        .send()
        .await
        .map_err(|e| Error::AwsError(e.into()))?;

    Ok(())
}

/// Downloads trust CSV data from S3 using "trust/{id}" key pattern and saves to file.
pub async fn download_trust_data_to_file(
    s3_client: &S3Client,
    bucket_name: &str,
    trust_id: &str,
    file_path: &str,
) -> Result<(), Error> {
    let object_key = format!("trust/{}", trust_id);
    download_s3_object_to_file(s3_client, bucket_name, &object_key, file_path).await
}

/// Downloads seed CSV data from S3 using "seed/{id}" key pattern and saves to file.
pub async fn download_seed_data_to_file(
    s3_client: &S3Client,
    bucket_name: &str,
    seed_id: &str,
    file_path: &str,
) -> Result<(), Error> {
    let object_key = format!("seed/{}", seed_id);
    download_s3_object_to_file(s3_client, bucket_name, &object_key, file_path).await
}

/// Downloads JSON metadata from S3 using "meta/{id}" key pattern and parses it into the specified type.
pub async fn download_json_metadata_from_s3<T>(
    s3_client: &S3Client,
    bucket_name: &str,
    meta_id: &str,
) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let object_key = format!("meta/{}", meta_id);
    let mut response = s3_client
        .get_object()
        .bucket(bucket_name)
        .key(&object_key)
        .send()
        .await
        .map_err(|e| Error::AwsError(e.into()))?;

    let mut data = Vec::new();
    while let Some(bytes) = response.body.next().await {
        let chunk = bytes.map_err(Error::ByteStreamError)?;
        data.extend_from_slice(&chunk);
    }

    let metadata: T = serde_json::from_slice(&data).map_err(Error::SerdeError)?;
    Ok(metadata)
}

/// Parses CSV data from a file handle into TrustEntry vectors.
pub fn parse_trust_entries_from_file(
    file: std::fs::File,
) -> Result<Vec<openrank_common::TrustEntry>, Error> {
    let mut reader = csv::Reader::from_reader(file);
    let mut entries = Vec::new();

    for result in reader.records() {
        let record: csv::StringRecord = result.map_err(Error::CsvError)?;
        let (from, to, value): (String, String, f32) =
            record.deserialize(None).map_err(Error::CsvError)?;
        let trust_entry = openrank_common::TrustEntry::new(from, to, value);
        entries.push(trust_entry);
    }

    Ok(entries)
}

/// Parses CSV data from a file handle into ScoreEntry vectors.
pub fn parse_score_entries_from_file(
    file: std::fs::File,
) -> Result<Vec<openrank_common::ScoreEntry>, Error> {
    let mut reader = csv::Reader::from_reader(file);
    let mut entries = Vec::new();

    for result in reader.records() {
        let record: csv::StringRecord = result.map_err(Error::CsvError)?;
        let (id, value): (String, f32) = record.deserialize(None).map_err(Error::CsvError)?;
        let score_entry = openrank_common::ScoreEntry::new(id, value);
        entries.push(score_entry);
    }

    Ok(entries)
}
