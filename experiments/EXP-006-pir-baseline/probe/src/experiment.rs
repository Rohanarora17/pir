use crate::{engine::Engine, wire};
use alloy_primitives::keccak256;
use ikpir_client::HintPatchClient;
use ikpir_common::{FrodoPirBackend, PirQueryBundle, PirResponseBundle, SimplePirBackend};
use ikpir_server::Segmented2aryIkpirServer;
use pvrpc_record_geometry::{corpus, decode, encode};
use pvrpc_verifier::{StorageOutcome, Verifier};
use segmented_cuckoo::Segmented2aryCuckooKVStore;
use serde_json::{Value, json};
use std::{error::Error, path::Path, time::Instant};

type Fallible<T> = Result<T, Box<dyn Error>>;

pub fn run(directory: &Path, engine: &str, mode: &str, record_size: usize) -> Fallible<Value> {
    if ![3072, 4096].contains(&record_size) || !["index", "keyword"].contains(&mode) {
        return Err("supported cases: index|keyword, 3072|4096 bytes".into());
    }
    match engine {
        "simple" => run_engine::<SimplePirBackend>(directory, mode, record_size),
        "frodo" => run_engine::<FrodoPirBackend>(directory, mode, record_size),
        _ => Err("unknown PIR engine".into()),
    }
}

enum Service<B: Engine> {
    Index {
        db: Vec<u32>,
        params: B::ServerParams,
        client: B::ClientState,
        rows: u32,
        width: u32,
    },
    Keyword {
        server: Box<Segmented2aryIkpirServer<B>>,
        client: HintPatchClient<B>,
    },
}

struct Session<B: Engine> {
    service: Service<B>,
    binding: [u8; 32],
    query_widths: Vec<usize>,
    response_widths: Vec<usize>,
    setup: Value,
}

impl<B: Engine> Session<B> {
    fn new(
        frames: &[Vec<u8>],
        keys: &[[u8; 32]],
        manifest_bytes: &[u8],
        mode: &str,
    ) -> Fallible<Self> {
        let record_size = frames[0].len();
        let (mut setup, service, params) = if mode == "index" {
            let start = Instant::now();
            let db: Vec<u32> = frames
                .iter()
                .flat_map(|f| f.iter().map(|b| u32::from(*b)))
                .collect();
            let packing_us = start.elapsed().as_micros();
            let rows = frames.len() as u32;
            let width = record_size as u32;
            // The correctness bound sees exactly width*8 bits, with one nominal bit in
            // the helper's fingerprint term. Direct mode stores no fingerprints.
            let max_pb = B::max_plaintext_bits(1, rows, 1, 1, width * 8 - 1);
            if max_pb < 8 {
                return Err("8-bit cells exceed the derived correctness bound".into());
            }
            let start = Instant::now();
            let (params, _material, hint) =
                B::server_setup(&B::Config::default(), &db, rows, width, 8);
            let server_setup_us = start.elapsed().as_micros();
            let hint_bytes = B::hint_byte_size(&hint);
            let setup_min = B::server_params_byte_size(&params) + hint_bytes + 32;
            let shape = B::db_matrix_shape(&params);
            let start = Instant::now();
            let client = B::client_setup(&params, &hint);
            let client_setup_us = start.elapsed().as_micros();
            let setup = json!({"packing_us":packing_us,"server_setup_us":server_setup_us,"bundle_export_us":0,
                "client_setup_us":client_setup_us,"setup_payload_min_bytes":setup_min,"hint_bytes":hint_bytes,
                "client_public_arrays_bytes":hint_bytes+shape.0 as usize*B::LWE_DIM as usize*4,
                "server_database_cell_bytes":db.len()*4,"logical_slots":frames.len(),"segments":1,
                "bucket_size":1,"fingerprint_bits":0,"max_plaintext_bits_from_upstream_bound":max_pb});
            let public = vec![params.clone()];
            (
                setup,
                Service::Index {
                    db,
                    params,
                    client,
                    rows,
                    width,
                },
                public,
            )
        } else {
            let max_pb = B::max_plaintext_bits(2, 8, 4, 64, record_size as u32 * 8);
            if max_pb < 8 {
                return Err("keyword 8-bit cells exceed the correctness bound".into());
            }
            let start = Instant::now();
            let mut store = Segmented2aryCuckooKVStore::new(16, 4, 64, record_size as u32 * 8, 8)?;
            for (key, frame) in keys.iter().zip(frames) {
                store.insert(key, frame)?;
            }
            let database_bytes = store.size_in_bytes();
            let packing_us = start.elapsed().as_micros();
            let start = Instant::now();
            let server = Segmented2aryIkpirServer::<B>::new(store, B::Config::default());
            let server_setup_us = start.elapsed().as_micros();
            let start = Instant::now();
            let bundle = server.setup();
            let bundle_export_us = start.elapsed().as_micros();
            let hint_bytes: usize = bundle.hints.iter().map(B::hint_byte_size).sum();
            let setup_min = bundle.wire_byte_size() + 32;
            let public = bundle.backend_params.clone();
            let material_bytes: usize = public
                .iter()
                .map(|p| B::db_matrix_shape(p).0 as usize * B::LWE_DIM as usize * 4)
                .sum();
            let start = Instant::now();
            let client = HintPatchClient::<B>::from_setup(bundle);
            let client_setup_us = start.elapsed().as_micros();
            let setup = json!({"packing_us":packing_us,"server_setup_us":server_setup_us,"bundle_export_us":bundle_export_us,
                "client_setup_us":client_setup_us,"setup_payload_min_bytes":setup_min,"hint_bytes":hint_bytes,
                "client_public_arrays_bytes":hint_bytes+material_bytes,"server_database_cell_bytes":database_bytes,
                "logical_slots":64,"segments":2,"bucket_size":4,"fingerprint_bits":64,
                "max_plaintext_bits_from_upstream_bound":max_pb});
            (
                setup,
                Service::Keyword {
                    server: Box::new(server),
                    client,
                },
                public,
            )
        };
        let shapes: Vec<(u32, u32)> = params.iter().map(B::db_matrix_shape).collect();
        let query_widths = shapes.iter().map(|p| p.0 as usize).collect();
        let response_widths = shapes.iter().map(|p| p.1 as usize).collect();
        setup["backend_matrix_shapes"] = json!(shapes);
        let mut context = manifest_bytes.to_vec();
        context.extend(B::NAME.as_bytes());
        context.extend(mode.as_bytes());
        // Seeds and actual geometry are public backend params; hash them with the snapshot
        // and database digest to distinguish independent preprocessing runs of one generation.
        context.extend(format!("{params:?}").as_bytes());
        context.extend(keccak256(frames.concat()).as_slice());
        let binding = keccak256(context).0;
        Ok(Self {
            service,
            binding,
            query_widths,
            response_widths,
            setup,
        })
    }

    fn query(&mut self, index: u32, key: &[u8]) -> Vec<B::Query> {
        match &mut self.service {
            Service::Index { client, .. } => vec![B::client_query(client, index)],
            Service::Keyword { client, .. } => client.build_query(key).queries,
        }
    }
    fn answer(&self, queries: Vec<B::Query>) -> Fallible<Vec<B::Response>> {
        match &self.service {
            Service::Index {
                db,
                params,
                rows,
                width,
                ..
            } => Ok(vec![B::server_answer(
                params,
                db,
                *rows,
                *width,
                &queries[0],
            )]),
            Service::Keyword { server, .. } => Ok(server
                .answer(&PirQueryBundle { epoch: 0, queries })?
                .responses),
        }
    }
    fn recover(&self, key: &[u8], responses: Vec<B::Response>) -> Fallible<Vec<u8>> {
        match &self.service {
            Service::Index { client, .. } => B::client_decode(client, &responses[0])
                .into_iter()
                .map(|c| u8::try_from(c).map_err(|_| "decoded cell exceeds a byte".into()))
                .collect(),
            Service::Keyword { client, .. } => client
                .decode(
                    key,
                    &PirResponseBundle {
                        epoch: 0,
                        responses,
                    },
                )?
                .ok_or_else(|| "keyword not found; this is not authenticated zero".into()),
        }
    }
}

fn run_engine<B: Engine>(directory: &Path, mode: &str, record_size: usize) -> Fallible<Value> {
    let corpus = corpus::load(directory, record_size)?;
    if corpus.records.len() != 32 {
        return Err("this experiment declares exactly 32 source records".into());
    }
    let mut frames = Vec::new();
    let mut keys = Vec::new();
    let mut index_map = Vec::new();
    for (context, record) in &corpus.records {
        Verifier::default().verify(context, &corpus.manifest, record)?;
        frames.push(encode(record, record_size)?);
        keys.push(*context.storage_key.as_bytes());
        index_map
            .push(json!({"storage_key":context.storage_key,"record_index":context.record_index}));
    }
    let manifest_bytes = serde_json::to_vec(&corpus.manifest)?;
    let map_bytes = serde_json::to_vec(&index_map)?.len();
    let mut session = Session::<B>::new(&frames, &keys, &manifest_bytes, mode)?;
    let mut measurements = Vec::new();
    let mut counts = (0, 0);
    let mut invalid = false;
    let mut crossed = false;
    let mut bad_response = false;
    for (index, ((context, _), key)) in corpus.records.iter().zip(&keys).enumerate() {
        let begin = Instant::now();
        let start = Instant::now();
        let queries = session.query(index as u32, key);
        let query_us = start.elapsed().as_micros();
        let start = Instant::now();
        let vectors: Vec<Vec<u32>> = queries.iter().map(|q| B::query_cells(q).to_vec()).collect();
        let query_frame = wire::encode(*b"PQ06", session.binding, &vectors)?;
        let query_encoding_us = start.elapsed().as_micros();
        let start = Instant::now();
        let received = wire::decode(
            &query_frame,
            *b"PQ06",
            session.binding,
            &session.query_widths,
        )?;
        let queries = received.into_iter().map(B::query_from).collect();
        let server_framing_us = start.elapsed().as_micros();
        let start = Instant::now();
        let responses = session.answer(queries)?;
        let answer_us = start.elapsed().as_micros();
        let start = Instant::now();
        let vectors: Vec<Vec<u32>> = responses
            .iter()
            .map(|r| B::response_cells(r).to_vec())
            .collect();
        let response_frame = wire::encode(*b"PR06", session.binding, &vectors)?;
        let response_encoding_us = start.elapsed().as_micros();
        let start = Instant::now();
        let received = wire::decode(
            &response_frame,
            *b"PR06",
            session.binding,
            &session.response_widths,
        )?;
        let responses = received.into_iter().map(B::response_from).collect();
        let client_framing_us = start.elapsed().as_micros();
        let start = Instant::now();
        let decoded = session.recover(key, responses)?;
        let pir_decode_us = start.elapsed().as_micros();
        let start = Instant::now();
        if decoded != frames[index] {
            return Err(format!("PIR decoded wrong row {index}").into());
        }
        let record = decode(&decoded, record_size)?;
        match Verifier::default().verify(context, &corpus.manifest, &record)? {
            StorageOutcome::Present(_) => counts.0 += 1,
            StorageOutcome::Absent => counts.1 += 1,
        }
        let verify_us = start.elapsed().as_micros();
        let total_us = begin.elapsed().as_micros();
        measurements.push(json!({"record_index":index,"query_us":query_us,"request_encoding_us":query_encoding_us,
            "server_frame_decoding_us":server_framing_us,"server_answer_us":answer_us,"response_encoding_us":response_encoding_us,
            "client_frame_decoding_us":client_framing_us,"pir_decode_us":pir_decode_us,"record_and_proof_verification_us":verify_us,
            "total_local_us":total_us,"request_bytes":query_frame.len(),"response_bytes":response_frame.len()}));
        // Admission checks run after timing. Invalid messages never reach low-level backend calls.
        if index == 0 {
            let mut malformed = query_frame.clone();
            malformed[40..44].copy_from_slice(&u32::MAX.to_be_bytes());
            invalid =
                wire::decode(&malformed, *b"PQ06", session.binding, &session.query_widths).is_err();
            let mut mismatch = query_frame;
            mismatch[4] ^= 1;
            crossed =
                wire::decode(&mismatch, *b"PQ06", session.binding, &session.query_widths).is_err();
            bad_response = wire::decode(
                &response_frame[..response_frame.len() - 1],
                *b"PR06",
                session.binding,
                &session.response_widths,
            )
            .is_err();
        }
    }
    if !invalid || !crossed || !bad_response {
        return Err("an admission check failed".into());
    }
    Ok(
        json!({"experiment_id":"EXP-006","engine":B::NAME,"mode":mode,"record_size_bytes":record_size,
        "records":frames.len(),"queries_completed":measurements.len(),"verified_inclusions":counts.0,"verified_absences":counts.1,
        "lwe_dimension":B::LWE_DIM,"simple_sigma":if B::NAME=="simple" {Some(6.4)} else {None},
        "plaintext_bits":8,"correctness_target_from_upstream_kappa":40,"setup":session.setup,
        "invalid_query_rejected":invalid,"cross_preprocessing_query_rejected":crossed,"truncated_response_rejected":bad_response,
        "public_manifest_bytes":manifest_bytes.len(),"public_logical_index_map_bytes":map_bytes,
        "full_padded_dataset_download_bytes":frames.len()*record_size+manifest_bytes.len()+map_bytes,
        "client_state_scope":"public hint plus expanded public matrix; excludes pending secret, allocator overhead and code",
        "query_policy":"one outstanding per client; no precompute queues; 32 sequential cold reads",
        "root_authentication":"not_performed","network_requests":0,"timing_scope":"in-process cold sequential reads; no transport latency",
        "setup_byte_scope":"upstream canonical minimum plus 32-byte binding; no setup serializer tested",
        "measurements":measurements}),
    )
}
