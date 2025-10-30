pub mod algos;
pub mod eigenda;
pub mod logs;
pub mod merkle;
pub mod runner;

use alloy_primitives::TxHash;
use alloy_rlp::{BufMut, Decodable, Encodable, Error as RlpError, Result as RlpResult};
use csv::StringRecord;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read};

pub fn format_hex(hex: String) -> String {
    if hex.len() < 8 {
        return format!("0x{}", hex);
    }

    let first_part = hex.get(..4).unwrap();
    let second_part = hex.get((hex.len() - 4)..).unwrap();

    format!("0x{}...{}", first_part, second_part)
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ScoreEntry {
    id: String,
    value: f32,
}

impl ScoreEntry {
    pub fn new(id: String, value: f32) -> Self {
        Self { id, value }
    }
}

impl Encodable for ScoreEntry {
    fn encode(&self, out: &mut dyn BufMut) {
        self.id.encode(out);
        out.put_f32(self.value);
    }
}

impl Decodable for ScoreEntry {
    fn decode(buf: &mut &[u8]) -> RlpResult<Self> {
        let id = String::decode(buf)?;
        let mut value_bytes = [0; 4];
        let size = buf
            .read(&mut value_bytes)
            .map_err(|_| RlpError::Custom("Failed to read bytes"))?;
        if size != 4 {
            return RlpResult::Err(RlpError::UnexpectedLength);
        }
        let value = f32::from_be_bytes(value_bytes);
        Ok(ScoreEntry { id, value })
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct TrustEntry {
    from: String,
    to: String,
    value: f32,
}

impl TrustEntry {
    pub fn new(from: String, to: String, value: f32) -> Self {
        Self { from, to, value }
    }
}

impl Encodable for TrustEntry {
    fn encode(&self, out: &mut dyn BufMut) {
        self.from.encode(out);
        self.to.encode(out);
        out.put_f32(self.value);
    }
}

impl Decodable for TrustEntry {
    fn decode(buf: &mut &[u8]) -> RlpResult<Self> {
        let from = String::decode(buf)?;
        let to = String::decode(buf)?;
        let mut value_bytes = [0; 4];
        let size = buf
            .read(&mut value_bytes)
            .map_err(|_| RlpError::Custom("Failed to read bytes"))?;
        if size != 4 {
            return RlpResult::Err(RlpError::UnexpectedLength);
        }
        let value = f32::from_be_bytes(value_bytes);
        Ok(TrustEntry { from, to, value })
    }
}

/// Common job description used across computer, challenger, and rxp modules
#[derive(Serialize, Deserialize, Clone)]
pub struct JobDescription {
    pub name: String,
    pub trust_id: String,
    pub seed_id: String,
    pub algo_id: u32,
    pub params: HashMap<String, String>,
}

impl JobDescription {
    pub fn new(
        name: String,
        trust_id: String,
        seed_id: String,
        algo_id: u32,
        params: HashMap<String, String>,
    ) -> Self {
        Self {
            name,
            trust_id,
            seed_id,
            algo_id,
            params,
        }
    }
}

/// Common job result used across computer, challenger, and rxp modules
#[derive(Serialize, Deserialize, Clone)]
pub struct JobResult {
    pub scores_id: String,
    pub commitment: String,
}

impl JobResult {
    pub fn new(scores_id: String, commitment: String) -> Self {
        Self {
            scores_id,
            commitment,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    request_tx_hash: Option<TxHash>,
    results_tx_hash: Option<TxHash>,
}

impl JobMetadata {
    pub fn new() -> Self {
        Self {
            request_tx_hash: None,
            results_tx_hash: None,
        }
    }

    pub fn set_request_tx_hash(&mut self, request_tx_hash: TxHash) {
        self.request_tx_hash = Some(request_tx_hash);
    }

    pub fn set_results_tx_hash(&mut self, results_tx_hash: TxHash) {
        self.results_tx_hash = Some(results_tx_hash);
    }

    pub fn has_request_tx(&self) -> bool {
        self.request_tx_hash.is_some()
    }

    pub fn has_results_tx(&self) -> bool {
        self.results_tx_hash.is_some()
    }
}

/// Helper function to parse trust entries from a CSV file
pub fn parse_trust_entries_from_file(file: File) -> Result<Vec<TrustEntry>, csv::Error> {
    let mut reader = csv::Reader::from_reader(file);
    let mut entries = Vec::new();

    for result in reader.records() {
        let record: StringRecord = result?;
        let (from, to, value): (String, String, f32) = record.deserialize(None)?;
        let trust_entry = TrustEntry::new(from, to, value);
        entries.push(trust_entry);
    }

    Ok(entries)
}

/// Helper function to parse score entries from a CSV file
pub fn parse_score_entries_from_file(file: File) -> Result<Vec<ScoreEntry>, csv::Error> {
    let mut reader = csv::Reader::from_reader(file);
    let mut entries = Vec::new();

    for result in reader.records() {
        let record: StringRecord = result?;
        let (id, value): (String, f32) = record.deserialize(None)?;
        let score_entry = ScoreEntry::new(id, value);
        entries.push(score_entry);
    }

    Ok(entries)
}
