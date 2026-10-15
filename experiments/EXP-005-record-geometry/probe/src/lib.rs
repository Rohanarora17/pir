//! Experimental framing only. This format has not been accepted as a production protocol.
pub mod corpus;
use pvrpc_protocol::{Hash32, HexBytes, ProofRecord, RECORD_SCHEMA_VERSION, SnapshotRef};
use std::fmt;

const MAX_FRAME_BYTES: usize = 1024 * 1024;
const MAX_NODES: usize = 128;
const MAX_NODE_BYTES: usize = 4096;
const FIXED_FRAME_BYTES: usize = 174;
const MAGIC: &[u8; 4] = b"PVR0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    InvalidSize,
    TooLarge { needed: usize, capacity: usize },
    Malformed,
    UnsupportedSchema,
    InvalidProofShape,
    NonzeroPadding,
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "candidate frame: {self:?}")
    }
}
impl std::error::Error for CodecError {}

pub fn encoded_len(record: &ProofRecord) -> Result<usize, CodecError> {
    if record.schema_version != RECORD_SCHEMA_VERSION {
        return Err(CodecError::UnsupportedSchema);
    }
    if record.storage_proof.is_empty() || record.storage_proof.len() > MAX_NODES {
        return Err(CodecError::InvalidProofShape);
    }
    let mut size = FIXED_FRAME_BYTES;
    for node in &record.storage_proof {
        if node.as_bytes().is_empty() || node.as_bytes().len() > MAX_NODE_BYTES {
            return Err(CodecError::InvalidProofShape);
        }
        size += 4 + node.as_bytes().len(); // Bounded count and length prevent arithmetic overflow.
    }
    Ok(size)
}

pub fn encode(record: &ProofRecord, capacity: usize) -> Result<Vec<u8>, CodecError> {
    check_capacity(capacity)?;
    let needed = encoded_len(record)?;
    if needed > capacity {
        return Err(CodecError::TooLarge { needed, capacity });
    }
    let mut frame = Vec::with_capacity(capacity);
    frame.extend(MAGIC);
    frame.extend(((needed - 8) as u32).to_be_bytes());
    frame.extend(record.schema_version.to_be_bytes());
    frame.extend(record.snapshot.chain_id.to_be_bytes());
    frame.extend(record.snapshot.block_number.to_be_bytes());
    frame.extend(record.snapshot.block_hash.as_bytes());
    frame.extend(record.snapshot.state_root.as_bytes());
    frame.extend(record.snapshot.generation.to_be_bytes());
    frame.extend(record.record_index.to_be_bytes());
    frame.extend(record.storage_key.as_bytes());
    frame.extend(record.value.as_bytes());
    frame.extend((record.storage_proof.len() as u16).to_be_bytes());
    for node in &record.storage_proof {
        frame.extend((node.as_bytes().len() as u32).to_be_bytes());
        frame.extend(node.as_bytes());
    }
    frame.resize(capacity, 0);
    Ok(frame)
}

/// Parse exactly the caller's selected row size. This does not authenticate the decoded record.
pub fn decode(frame: &[u8], capacity: usize) -> Result<ProofRecord, CodecError> {
    check_capacity(capacity)?;
    if frame.len() != capacity || frame.get(..4) != Some(MAGIC) {
        return Err(CodecError::Malformed);
    }
    let mut outer = Reader(&frame[4..]);
    let length = outer.u32()? as usize;
    let end = 8usize.checked_add(length).ok_or(CodecError::Malformed)?;
    let payload = frame.get(8..end).ok_or(CodecError::Malformed)?;
    if frame[end..].iter().any(|byte| *byte != 0) {
        return Err(CodecError::NonzeroPadding);
    }
    let mut reader = Reader(payload);
    let schema_version = reader.u32()?;
    if schema_version != RECORD_SCHEMA_VERSION {
        return Err(CodecError::UnsupportedSchema);
    }
    let snapshot = SnapshotRef {
        chain_id: reader.u64()?,
        block_number: reader.u64()?,
        block_hash: Hash32::new(reader.array()?),
        state_root: Hash32::new(reader.array()?),
        generation: reader.u64()?,
    };
    let record_index = reader.u64()?;
    let storage_key = Hash32::new(reader.array()?);
    let value = Hash32::new(reader.array()?);
    let count = u16::from_be_bytes(reader.array()?) as usize;
    if count == 0 || count > MAX_NODES {
        return Err(CodecError::InvalidProofShape);
    }
    let mut storage_proof = Vec::with_capacity(count);
    for _ in 0..count {
        let length = reader.u32()? as usize;
        if length == 0 || length > MAX_NODE_BYTES {
            return Err(CodecError::InvalidProofShape);
        }
        storage_proof.push(HexBytes::new(reader.take(length)?.to_vec()));
    }
    if !reader.0.is_empty() {
        return Err(CodecError::Malformed);
    }
    Ok(ProofRecord {
        schema_version,
        snapshot,
        record_index,
        storage_key,
        value,
        storage_proof,
    })
}

fn check_capacity(capacity: usize) -> Result<(), CodecError> {
    if !(8..=MAX_FRAME_BYTES).contains(&capacity) {
        return Err(CodecError::InvalidSize);
    }
    Ok(())
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], CodecError> {
        let bytes = self.0.get(..length).ok_or(CodecError::Malformed)?;
        self.0 = &self.0[length..];
        Ok(bytes)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], CodecError> {
        self.take(N)?.try_into().map_err(|_| CodecError::Malformed)
    }
    fn u32(&mut self) -> Result<u32, CodecError> {
        Ok(u32::from_be_bytes(self.array()?))
    }
    fn u64(&mut self) -> Result<u64, CodecError> {
        Ok(u64::from_be_bytes(self.array()?))
    }
}
