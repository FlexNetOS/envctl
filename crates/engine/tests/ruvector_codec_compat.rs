#![cfg(feature = "ruvector-pg")]

//! Golden compatibility tests for the maintained codec forks used by the RuVector patches.
//!
//! Every byte string below was generated once from the exact unmodified crates.io releases named
//! in `third_party/ADVISORY-PATCHES.md`. The retired codecs are deliberately not dev-dependencies:
//! putting them back in the workspace graph would restore the advisories this test guards against.

use bincode_reloaded::{Decode, Encode};
use ruvector_core::index::hnsw::HnswIndex;
use ruvector_core::index::VectorIndex;
use ruvector_core::types::HnswConfig;
use ruvector_core::DistanceMetric;
use ruvector_graph::{Hyperedge, Node, NodeBuilder};
use ruvector_postgres::workers::engine::SearchResult;
use ruvllm::lora::adapters::trainer::{AdapterDataset, TrainingExample};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Encode, Decode)]
enum DirectFixture {
    Empty,
    Full {
        signed: i128,
        unsigned: u128,
        float_bits: u64,
        text: String,
        optional: Option<Vec<i32>>,
        result: Result<u64, String>,
        map: BTreeMap<String, Vec<u8>>,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum CborMode {
    Empty,
    Full { code: i64, label: String },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CborFixture {
    name: String,
    signed: i64,
    unsigned: u64,
    float_bits: u64,
    optional: Option<Vec<i32>>,
    mode: CborMode,
    map: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BorrowedCbor<'a> {
    text: &'a str,
}

fn decode_hex(input: &str) -> Vec<u8> {
    assert_eq!(input.len() % 2, 0);
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("golden hex is ASCII");
            u8::from_str_radix(text, 16).expect("golden hex is valid")
        })
        .collect()
}

fn direct_fixture() -> DirectFixture {
    DirectFixture::Full {
        signed: i128::MIN + 17,
        unsigned: u128::MAX - 23,
        float_bits: f64::from_bits(0x7ff8_1234_5678_9abc).to_bits(),
        text: "bincode wire compatibility: λ🦀".to_owned(),
        optional: Some(vec![i32::MIN, -1, 0, 1, i32::MAX]),
        result: Err("persisted error".to_owned()),
        map: BTreeMap::from([
            ("empty".to_owned(), Vec::new()),
            ("payload".to_owned(), vec![0, 1, 127, 128, 255]),
        ]),
    }
}

fn cbor_fixture() -> CborFixture {
    CborFixture {
        name: "pgrx PostgresType: λ🦀".to_owned(),
        signed: i64::MIN + 19,
        unsigned: u64::MAX - 29,
        float_bits: f64::from_bits(0x7ff8_1234_5678_9abc).to_bits(),
        optional: Some(vec![i32::MIN, -1, 0, 1, i32::MAX]),
        mode: CborMode::Full {
            code: -9_876_543_210,
            label: "persisted".to_owned(),
        },
        map: BTreeMap::from([
            ("empty".to_owned(), Vec::new()),
            ("payload".to_owned(), vec![0, 1, 127, 128, 255]),
        ]),
    }
}

#[test]
fn bincode_reloaded_matches_original_direct_wire_format() {
    let fixture = direct_fixture();
    let old_bytes = decode_hex(
        "01feddfffffffffffffffffffffffffffffffee8fffffffffffffffffffffffffffffffdbc9a78563412f87f2262696e636f6465207769726520636f6d7061746962696c6974793a20cebbf09fa6800105fcffffffff010002fcfeffffff010f706572736973746564206572726f720205656d70747900077061796c6f61640500017f80ff",
    );

    let new_bytes = bincode_reloaded::encode_to_vec(&fixture, bincode_reloaded::config::standard())
        .expect("encode maintained direct fixture");
    assert_eq!(new_bytes, old_bytes);

    let (decoded, consumed): (DirectFixture, usize) =
        bincode_reloaded::decode_from_slice(&old_bytes, bincode_reloaded::config::standard())
            .expect("maintained direct decoder reads original bytes");
    assert_eq!(consumed, old_bytes.len());
    assert_eq!(decoded, fixture);
}

#[test]
fn bincode_reloaded_preserves_actual_core_hnsw_bytes() {
    let old_bytes = decode_hex(
        "0108766563746f722d31030000a03f000020c0000070400108766563746f722d3100010008766563746f722d3101040c08100300",
    );
    let restored = HnswIndex::deserialize(&old_bytes).expect("read original core HNSW bytes");
    assert_eq!(
        restored.serialize().expect("re-serialize restored HNSW"),
        old_bytes
    );

    let mut fresh = HnswIndex::new(
        3,
        DistanceMetric::Euclidean,
        HnswConfig {
            m: 4,
            ef_construction: 12,
            ef_search: 8,
            max_elements: 16,
        },
    )
    .expect("construct maintained HNSW fixture");
    fresh
        .add("vector-1".to_owned(), vec![1.25, -2.5, 3.75])
        .expect("populate maintained HNSW fixture");
    assert_eq!(fresh.serialize().expect("serialize fresh HNSW"), old_bytes);
}

#[test]
fn bincode_reloaded_preserves_actual_graph_bytes() {
    let old_node = decode_hex("066e6f64652d310106506572736f6e01036167650254");
    let expected_node = NodeBuilder::new()
        .id("node-1")
        .label("Person")
        .property("age", 42_i64)
        .build();
    assert_eq!(
        bincode_reloaded::encode_to_vec(&expected_node, bincode_reloaded::config::standard())
            .expect("encode maintained graph node"),
        old_node
    );
    let (decoded, consumed): (Node, usize) =
        bincode_reloaded::decode_from_slice(&old_node, bincode_reloaded::config::standard())
            .expect("decode original graph node");
    assert_eq!(consumed, old_node.len());
    assert_eq!(decoded.id, "node-1");
    assert!(decoded.has_label("Person"));
    assert_eq!(
        decoded.get_property("age"),
        expected_node.get_property("age")
    );

    let old_hyperedge = decode_hex(
        "0768797065722d3102066e6f64652d31066e6f64652d32074d454554494e47010c77697265206669787475726501047965617202fbd40f0000403f",
    );
    let mut expected_hyperedge = Hyperedge::with_id(
        "hyper-1".to_owned(),
        vec!["node-1".to_owned(), "node-2".to_owned()],
        "MEETING",
    );
    expected_hyperedge
        .set_description("wire fixture")
        .set_confidence(0.75)
        .set_property("year", 2026_i64);
    assert_eq!(
        bincode_reloaded::encode_to_vec(&expected_hyperedge, bincode_reloaded::config::standard(),)
            .expect("encode maintained hyperedge"),
        old_hyperedge
    );
    let (decoded, consumed): (Hyperedge, usize) =
        bincode_reloaded::decode_from_slice(&old_hyperedge, bincode_reloaded::config::standard())
            .expect("decode original hyperedge");
    assert_eq!(consumed, old_hyperedge.len());
    assert_eq!(decoded.id, expected_hyperedge.id);
    assert_eq!(decoded.nodes, expected_hyperedge.nodes);
    assert_eq!(decoded.description, expected_hyperedge.description);
    assert_eq!(
        decoded.confidence.to_bits(),
        expected_hyperedge.confidence.to_bits()
    );
}

#[test]
fn bincode_reloaded_preserves_actual_ruvllm_serde_bytes() {
    let old_bytes = decode_hex(
        "01030000a03f000020c0000070400103000080400000a0400000c0400000603f010d636f6d7061746962696c697479010d73657269616c697a6174696f6e000c776972652d666978747572652761637475616c20706572736973746564207275766c6c6d2061646170746572206461746173657403",
    );
    let mut dataset = AdapterDataset::new("wire-fixture", 3);
    dataset.description = "actual persisted ruvllm adapter dataset".to_owned();
    dataset.add_example(
        TrainingExample::new(vec![1.25, -2.5, 3.75], 0.875)
            .with_target(vec![4.0, 5.0, 6.0])
            .with_task("compatibility")
            .with_domain("serialization"),
    );

    let new_bytes =
        bincode_reloaded::serde::encode_to_vec(&dataset, bincode_reloaded::config::standard())
            .expect("encode maintained ruvllm dataset");
    assert_eq!(new_bytes, old_bytes);
    let (decoded, consumed): (AdapterDataset, usize) = bincode_reloaded::serde::decode_from_slice(
        &old_bytes,
        bincode_reloaded::config::standard(),
    )
    .expect("decode original ruvllm dataset");
    assert_eq!(consumed, old_bytes.len());
    assert_eq!(decoded.name, dataset.name);
    assert_eq!(decoded.description, dataset.description);
    assert_eq!(decoded.examples[0].input, dataset.examples[0].input);
}

#[test]
fn fugue_bincode_preserves_postgres_state_and_hnsw_v2_payloads() {
    let old_search_result = decode_hex(
        "0300000000000000ffffffffffffffff0000000000000000ffffffffffffff7f0300000000000000000000000000a0bf0000807ffeffffffffffffff393000000000000001",
    );
    let result = SearchResult {
        ids: vec![-1, 0, i64::MAX],
        distances: vec![0.0, -1.25, f32::INFINITY],
        search_time_us: u64::MAX - 1,
        vectors_scanned: 12_345,
        cache_hit: true,
    };
    assert_eq!(result.to_bytes(), old_search_result);
    let decoded =
        SearchResult::from_bytes(&old_search_result).expect("decode original postgres state");
    assert_eq!(decoded.ids, result.ids);
    assert_eq!(decoded.distances, result.distances);
    assert_eq!(decoded.search_time_us, result.search_time_us);
    assert_eq!(decoded.vectors_scanned, result.vectors_scanned);
    assert_eq!(decoded.cache_hit, result.cache_hit);

    // hnsw_rs format-v2 stores each vector as a bincode-1 payload. Keeping this branch working is
    // mandatory; the upstream proposal that exits on format v2 would be a capability downgrade.
    let old_hnsw_payload = decode_hex("04000000000000000000a03f000020c0000070400000807f");
    let payload = vec![1.25_f32, -2.5, 3.75, f32::INFINITY];
    assert_eq!(
        fugue_bincode::serialize(&payload).expect("encode maintained HNSW v2 payload"),
        old_hnsw_payload
    );
    let decoded: Vec<f32> =
        fugue_bincode::deserialize(&old_hnsw_payload).expect("decode original HNSW v2 payload");
    assert_eq!(decoded, payload);
}

#[test]
fn serde_cbor_2_preserves_pgrx_cbor_bytes_and_borrowing() {
    let fixture = cbor_fixture();
    let old_bytes = decode_hex(
        "a7646e616d6578197067727820506f737467726573547970653a20cebbf09fa680667369676e65643b7fffffffffffffec68756e7369676e65641bffffffffffffffe26a666c6f61745f626974731b7ff8123456789abc686f7074696f6e616c853a7fffffff2000011a7fffffff646d6f6465a16446756c6ca264636f64653b000000024cb016e9656c6162656c69706572736973746564636d6170a265656d70747980677061796c6f6164850001187f188018ff",
    );
    assert_eq!(
        serde_cbor_2::to_vec(&fixture).expect("encode maintained pgrx CBOR fixture"),
        old_bytes
    );
    assert_eq!(
        serde_cbor_2::from_slice::<CborFixture>(&old_bytes)
            .expect("decode original pgrx CBOR fixture"),
        fixture
    );

    let old_borrowed = decode_hex("a1647465787469626f72726f77206d65");
    let decoded: BorrowedCbor<'_> =
        serde_cbor_2::from_slice(&old_borrowed).expect("borrow from original CBOR bytes");
    assert_eq!(decoded.text, "borrow me");
    let start = old_borrowed.as_ptr() as usize;
    let end = start + old_borrowed.len();
    let pointer = decoded.text.as_ptr() as usize;
    assert!(
        (start..end).contains(&pointer),
        "decoded string must borrow the original input"
    );
}
