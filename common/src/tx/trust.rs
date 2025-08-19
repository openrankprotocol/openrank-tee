use alloy::hex::{self, FromHex};
use alloy_primitives::Address;
use alloy_rlp::{BufMut, Decodable, Encodable, Error as RlpError, Result as RlpResult};
use alloy_rlp_derive::{RlpDecodable, RlpEncodable};
use core::result::Result as CoreResult;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Read;

#[derive(
    Debug, Clone, Hash, Default, PartialEq, Eq, RlpDecodable, RlpEncodable, Serialize, Deserialize,
)]
pub struct OwnedNamespace(#[serde(with = "hex")] [u8; 24]);

impl OwnedNamespace {
    pub fn new(owner: Address, id: u32) -> Self {
        let mut bytes = [0; 24];
        bytes[..20].copy_from_slice(owner.as_slice());
        bytes[20..24].copy_from_slice(&id.to_be_bytes());
        Self(bytes)
    }

    pub fn to_hex(self) -> String {
        hex::encode(self.0)
    }

    pub fn owner(&self) -> Address {
        let mut bytes = [0; 20];
        bytes.copy_from_slice(&self.0[..20]);
        Address::from_slice(&bytes)
    }

    pub fn id(&self) -> u32 {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(&self.0[20..]);
        u32::from_be_bytes(bytes)
    }

    pub fn inner(&self) -> &[u8; 24] {
        &self.0
    }
}

impl Display for OwnedNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:#}:{}", self.owner(), self.id())
    }
}

impl FromHex for OwnedNamespace {
    type Error = hex::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> CoreResult<Self, Self::Error> {
        Ok(OwnedNamespace(<[u8; 24]>::from_hex(hex)?))
    }
}

#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, RlpEncodable, RlpDecodable, Getters,
)]
#[getset(get = "pub")]
#[rlp(trailing)]
pub struct TrustUpdate {
    trust_id: OwnedNamespace,
    entries: Vec<TrustEntry>,
    seq_number: Option<u64>,
}

impl TrustUpdate {
    pub fn new(trust_id: OwnedNamespace, entries: Vec<TrustEntry>) -> Self {
        Self {
            trust_id,
            entries,
            seq_number: None,
        }
    }
}

#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, RlpEncodable, RlpDecodable, Getters,
)]
#[getset(get = "pub")]
#[rlp(trailing)]
pub struct SeedUpdate {
    seed_id: OwnedNamespace,
    entries: Vec<ScoreEntry>,
    seq_number: Option<u64>,
}

impl SeedUpdate {
    pub fn new(seed_id: OwnedNamespace, entries: Vec<ScoreEntry>) -> Self {
        Self {
            seed_id,
            entries,
            seq_number: None,
        }
    }
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
