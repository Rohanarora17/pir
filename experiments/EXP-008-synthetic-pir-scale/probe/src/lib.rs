use ikpir_common::backend::simple::{SimpleQuery, SimpleResponse};
use ikpir_common::{BackendWireSize, IndexPirBackend, SimplePirBackend as B};
use pvrpc_pir_baseline::wire;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{error::Error, time::Instant};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const SEED: u64 = 0x505652504353434c;
const N: u64 = 1275;

/// Synthetic payload generator only. Never used for PIR secrets or public-matrix seeds.
pub fn cell(row: u32, offset: u32) -> u32 {
    let mut z = SEED.wrapping_add((u64::from(row) << 32) | u64::from(offset));
    z = z.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((z ^ (z >> 31)) & 255) as u32
}

/// Mirrors private upstream reshape_dims at the pinned revision. Run-time setup checks it.
pub fn plan(rows: u32, width: u32, budget: u64) -> Result<Value> {
    if rows == 0 || rows > (1 << 24) || ![3072, 4096].contains(&width) {
        return Err("rows must be 1..=2^24 and width 3072 or 4096".into());
    }
    let k = (f64::from(rows) / f64::from(width)).sqrt().round().max(1.0) as u32;
    let r = rows.div_ceil(k);
    let c = k.checked_mul(width).ok_or("reshape width overflow")?;
    let db_bytes = u64::from(rows) * u64::from(width) * 4;
    let matrix_bytes = u64::from(r) * N * 4;
    let hint_bytes = u64::from(c) * N * 4;
    let estimate = (db_bytes + 2 * matrix_bytes + 2 * hint_bytes) * 5 / 4 + (64 << 20);
    let max_pb =
        ikpir_common::pir_params::simple_max_plaintext_bits(1, rows, 1, 1, width * 8 - 1, 6.4);
    let reason = if max_pb < 8 {
        Some("correctness_bound")
    } else if r > 65536 || c > 65536 {
        Some("existing_wire_width_limit")
    } else if estimate > budget {
        Some("estimated_memory_budget")
    } else {
        None
    };
    Ok(
        json!({"rows":rows,"record_bytes":width,"reshape_k":k,"matrix_rows":r,"matrix_columns":c,
        "database_payload_bytes":u64::from(rows)*u64::from(width),"server_u32_database_bytes":db_bytes,
        "public_matrix_bytes":matrix_bytes,"hint_bytes":hint_bytes,
        "estimated_peak_working_bytes":estimate,"memory_budget_bytes":budget,
        "max_plaintext_bits":max_pb,"admitted":reason.is_none(),"skip_reason":reason}),
    )
}

pub fn targets(rows: u32, count: u32) -> Vec<u32> {
    (0..count)
        .map(|i| match i {
            0 => 0,
            1 => rows - 1,
            2 => (rows - 1).min(1),
            3 => rows.saturating_sub(2),
            _ => ((u64::from(i - 3) * 2654435761) % u64::from(rows)) as u32,
        })
        .collect()
}

pub fn run(rows: u32, width: u32, budget: u64, count: u32) -> Result<Value> {
    if !(4..=1024).contains(&count) {
        return Err("query count must be 4..=1024".into());
    }
    let geometry = plan(rows, width, budget)?;
    if geometry["admitted"] != true {
        return Ok(json!({"experiment_id":"EXP-008","status":"skipped","plan":geometry}));
    }
    let begin = Instant::now();
    let mut hash = Sha256::new();
    let mut db = Vec::with_capacity(rows as usize * width as usize);
    for row in 0..rows {
        let bytes: Vec<u8> = (0..width).map(|offset| cell(row, offset) as u8).collect();
        hash.update(&bytes);
        db.extend(bytes.into_iter().map(u32::from));
    }
    let database_sha256 = format!("{:x}", hash.finalize());
    let generation_us = begin.elapsed().as_micros();
    let begin = Instant::now();
    let (params, material, hint) = B::server_setup(&Default::default(), &db, rows, width, 8);
    let server_setup_us = begin.elapsed().as_micros();
    let (r, c) = B::db_matrix_shape(&params);
    if geometry["matrix_rows"] != r
        || geometry["matrix_columns"] != c
        || params.params.lwe_dim != N as u32
        || params.params.sigma != 6.4
        || params.params.plaintext_bits != 8
        || geometry["hint_bytes"] != B::hint_byte_size(&hint)
    {
        return Err("setup differs from preflight or fixed parameters".into());
    }
    let setup_min = B::server_params_byte_size(&params) + B::hint_byte_size(&hint) + 32;
    let begin = Instant::now();
    let client = B::client_setup(&params, &hint);
    let client_setup_us = begin.elapsed().as_micros();
    // Read-only server needs neither retained setup matrix nor a second hint copy.
    drop(material);
    drop(hint);
    let mut client = client;
    let binding: [u8; 32] =
        Sha256::digest(format!("EXP-008:{rows}:{width}:{params:?}:{database_sha256}").as_bytes())
            .into();
    let mut measurements = Vec::new();
    for index in targets(rows, count) {
        let begin = Instant::now();
        let start = Instant::now();
        let q = B::client_query(&mut client, index);
        let query_us = start.elapsed().as_micros();
        let start = Instant::now();
        let frame = wire::encode(*b"PQ06", binding, &[q.b])?;
        let mut vectors = wire::decode(&frame, *b"PQ06", binding, &[r as usize])?;
        let query = SimpleQuery {
            b: vectors.remove(0),
        };
        let request_framing_us = start.elapsed().as_micros();
        let start = Instant::now();
        let response = B::server_answer(&params, &db, rows, width, &query);
        let answer_us = start.elapsed().as_micros();
        let start = Instant::now();
        let reply = wire::encode(*b"PR06", binding, &[response.a])?;
        let mut vectors = wire::decode(&reply, *b"PR06", binding, &[c as usize])?;
        let response = SimpleResponse {
            a: vectors.remove(0),
        };
        let response_framing_us = start.elapsed().as_micros();
        let start = Instant::now();
        let recovered = B::client_decode(&client, &response);
        let decode_us = start.elapsed().as_micros();
        let total_us = begin.elapsed().as_micros();
        if recovered.len() != width as usize
            || recovered
                .iter()
                .enumerate()
                .any(|(offset, b)| *b != cell(index, offset as u32))
        {
            return Err(format!("incorrect synthetic row {index}").into());
        }
        measurements.push(json!({"record_index":index,"query_us":query_us,
            "request_framing_us":request_framing_us,"server_answer_us":answer_us,
            "response_framing_us":response_framing_us,"decode_us":decode_us,
            "total_local_us":total_us,"request_bytes":frame.len(),"response_bytes":reply.len()}));
    }
    Ok(
        json!({"experiment_id":"EXP-008","status":"completed","claim_type":"synthetic result",
        "plan":geometry,"database_sha256":database_sha256,"generator":"SplitMix64-v1 row/offset low byte",
        "generator_seed_hex":format!("{SEED:016x}"),"generation_us":generation_us,
        "server_setup_us":server_setup_us,"client_setup_us":client_setup_us,
        "setup_payload_min_bytes":setup_min,"client_public_arrays_bytes":u64::from(r+c)*N*4,
        "security":{"lwe_dimension":N,"sigma":6.4,"ciphertext_modulus_log2":32,
            "plaintext_bits":8,"upstream_correctness_kappa":40},
        "queries_completed":measurements.len(),"measurements":measurements,
        "network_requests":0,"proof_verification":"not_applicable_synthetic_bytes",
        "timing_scope":"local query/framing/answer/decode; excludes expected-byte comparison and network",
        "setup_scope":"canonical minimum estimate plus binding; not serialized or network transferred"}),
    )
}
