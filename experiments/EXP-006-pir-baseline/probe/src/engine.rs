use ikpir_common::backend::{
    frodo::{FrodoQuery, FrodoResponse},
    simple::{SimpleQuery, SimpleResponse},
};
use ikpir_common::{BackendWireSize, FrodoPirBackend, IndexPirBackend, SimplePirBackend};

/// Small experimental bridge; no cryptographic primitive is implemented here.
pub(crate) trait Engine:
    IndexPirBackend<Query: Clone, Response: Clone, ServerParams: std::fmt::Debug> + BackendWireSize
{
    const NAME: &'static str;
    const LWE_DIM: u32;
    fn max_plaintext_bits(arity: u32, rows: u32, bucket: u32, fingerprint: u32, value: u32) -> u32;
    fn query_cells(query: &Self::Query) -> &[u32];
    fn query_from(cells: Vec<u32>) -> Self::Query;
    fn response_cells(response: &Self::Response) -> &[u32];
    fn response_from(cells: Vec<u32>) -> Self::Response;
}
impl Engine for FrodoPirBackend {
    const NAME: &'static str = "frodo";
    const LWE_DIM: u32 = 1566;
    fn max_plaintext_bits(a: u32, r: u32, b: u32, f: u32, v: u32) -> u32 {
        ikpir_common::pir_params::frodo_max_plaintext_bits(a, r, b, f, v)
    }
    fn query_cells(q: &Self::Query) -> &[u32] {
        &q.b
    }
    fn query_from(b: Vec<u32>) -> Self::Query {
        FrodoQuery { b }
    }
    fn response_cells(r: &Self::Response) -> &[u32] {
        &r.a
    }
    fn response_from(a: Vec<u32>) -> Self::Response {
        FrodoResponse { a }
    }
}
impl Engine for SimplePirBackend {
    const NAME: &'static str = "simple";
    const LWE_DIM: u32 = 1275;
    fn max_plaintext_bits(a: u32, r: u32, b: u32, f: u32, v: u32) -> u32 {
        ikpir_common::pir_params::simple_max_plaintext_bits(a, r, b, f, v, 6.4)
    }
    fn query_cells(q: &Self::Query) -> &[u32] {
        &q.b
    }
    fn query_from(b: Vec<u32>) -> Self::Query {
        SimpleQuery { b }
    }
    fn response_cells(r: &Self::Response) -> &[u32] {
        &r.a
    }
    fn response_from(a: Vec<u32>) -> Self::Response {
        SimpleResponse { a }
    }
}
