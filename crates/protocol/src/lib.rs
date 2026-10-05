use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const RECORD_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hash32([u8; 32]);

impl Hash32 {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address([u8; 20]);

impl Address {
    pub const fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    fn is_zero(&self) -> bool {
        self.0 == [0; 20]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HexBytes(Vec<u8>);

impl HexBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(2 + bytes.len() * 2);
    encoded.push_str("0x");
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_hex<E: de::Error>(value: &str) -> Result<Vec<u8>, E> {
    let digits = value
        .strip_prefix("0x")
        .ok_or_else(|| E::custom("hex value must start with 0x"))?;
    if digits.len() % 2 != 0 {
        return Err(E::custom("hex value must contain a whole number of bytes"));
    }

    digits
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = decode_nibble(pair[0]).ok_or_else(|| E::custom("invalid hex digit"))?;
            let low = decode_nibble(pair[1]).ok_or_else(|| E::custom("invalid hex digit"))?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn decode_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

macro_rules! impl_fixed_hex_serde {
    ($type:ty, $length:expr, $name:literal) => {
        impl Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&encode_hex(&self.0))
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                let bytes = decode_hex::<D::Error>(&value)?;
                let actual = bytes.len();
                let bytes = bytes.try_into().map_err(|_| {
                    de::Error::custom(format_args!(
                        "{} must contain {} bytes, found {}",
                        $name, $length, actual
                    ))
                })?;
                Ok(Self(bytes))
            }
        }
    };
}

impl_fixed_hex_serde!(Hash32, 32, "hash");
impl_fixed_hex_serde!(Address, 20, "address");

impl Serialize for HexBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode_hex(&self.0))
    }
}

impl<'de> Deserialize<'de> for HexBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self(decode_hex::<D::Error>(&value)?))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotRef {
    pub chain_id: u64,
    pub block_number: u64,
    pub block_hash: Hash32,
    pub state_root: Hash32,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSpec {
    pub record_count: u64,
    pub record_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PirParameters {
    pub scheme: String,
    pub parameter_set: String,
    pub query_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotManifest {
    pub schema_version: u32,
    pub snapshot: SnapshotRef,
    pub contract: Address,
    pub mapping_slot: Hash32,
    pub dataset: DatasetSpec,
    pub pir: PirParameters,
    pub account_proof: Vec<HexBytes>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError {
    UnsupportedSchema { expected: u32, actual: u32 },
    ZeroChainId,
    ZeroGeneration,
    ZeroContract,
    EmptyDataset,
    ZeroRecordSize,
    EmptyPirScheme,
    EmptyParameterSet,
    ZeroQueryCount,
    MissingAccountProof,
    EmptyAccountProofNode { index: usize },
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { expected, actual } => write!(
                formatter,
                "unsupported manifest schema version {actual}, expected {expected}"
            ),
            Self::ZeroChainId => formatter.write_str("chain ID must be non-zero"),
            Self::ZeroGeneration => formatter.write_str("snapshot generation must be non-zero"),
            Self::ZeroContract => formatter.write_str("contract address must be non-zero"),
            Self::EmptyDataset => formatter.write_str("dataset has no records"),
            Self::ZeroRecordSize => formatter.write_str("record size must be non-zero"),
            Self::EmptyPirScheme => formatter.write_str("PIR scheme is empty"),
            Self::EmptyParameterSet => formatter.write_str("PIR parameter set is empty"),
            Self::ZeroQueryCount => formatter.write_str("PIR query count must be non-zero"),
            Self::MissingAccountProof => formatter.write_str("account proof is missing"),
            Self::EmptyAccountProofNode { index } => {
                write!(formatter, "account proof node {index} is empty")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

impl SnapshotManifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchema {
                expected: MANIFEST_SCHEMA_VERSION,
                actual: self.schema_version,
            });
        }
        if self.snapshot.chain_id == 0 {
            return Err(ManifestError::ZeroChainId);
        }
        if self.snapshot.generation == 0 {
            return Err(ManifestError::ZeroGeneration);
        }
        if self.contract.is_zero() {
            return Err(ManifestError::ZeroContract);
        }
        if self.dataset.record_count == 0 {
            return Err(ManifestError::EmptyDataset);
        }
        if self.dataset.record_size == 0 {
            return Err(ManifestError::ZeroRecordSize);
        }
        if self.pir.scheme.trim().is_empty() {
            return Err(ManifestError::EmptyPirScheme);
        }
        if self.pir.parameter_set.trim().is_empty() {
            return Err(ManifestError::EmptyParameterSet);
        }
        if self.pir.query_count == 0 {
            return Err(ManifestError::ZeroQueryCount);
        }
        if self.account_proof.is_empty() {
            return Err(ManifestError::MissingAccountProof);
        }
        if let Some(index) = self.account_proof.iter().position(HexBytes::is_empty) {
            return Err(ManifestError::EmptyAccountProofNode { index });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofRecord {
    pub schema_version: u32,
    pub snapshot: SnapshotRef,
    pub record_index: u64,
    pub storage_key: Hash32,
    pub value: Hash32,
    pub storage_proof: Vec<HexBytes>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordError {
    InvalidManifest(ManifestError),
    UnsupportedSchema { expected: u32, actual: u32 },
    SnapshotMismatch,
    RecordIndexOutOfBounds { index: u64, record_count: u64 },
    StorageKeyMismatch,
    MissingStorageProof,
    EmptyStorageProofNode { index: usize },
}

impl ProofRecord {
    pub fn validate_for(
        &self,
        manifest: &SnapshotManifest,
        requested_key: &Hash32,
    ) -> Result<(), RecordError> {
        manifest.validate().map_err(RecordError::InvalidManifest)?;

        if self.schema_version != RECORD_SCHEMA_VERSION {
            return Err(RecordError::UnsupportedSchema {
                expected: RECORD_SCHEMA_VERSION,
                actual: self.schema_version,
            });
        }
        if self.snapshot != manifest.snapshot {
            return Err(RecordError::SnapshotMismatch);
        }
        if self.record_index >= manifest.dataset.record_count {
            return Err(RecordError::RecordIndexOutOfBounds {
                index: self.record_index,
                record_count: manifest.dataset.record_count,
            });
        }
        if &self.storage_key != requested_key {
            return Err(RecordError::StorageKeyMismatch);
        }
        if self.storage_proof.is_empty() {
            return Err(RecordError::MissingStorageProof);
        }
        if let Some(index) = self.storage_proof.iter().position(HexBytes::is_empty) {
            return Err(RecordError::EmptyStorageProofNode { index });
        }

        Ok(())
    }
}
