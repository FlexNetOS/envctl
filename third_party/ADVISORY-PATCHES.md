# Advisory remediation patches

These directories are unpacked from the exact crates.io release archives listed below. They are
not snapshots of local checkouts and, in particular, are not sourced from the stale
`meta-ruvector` checkout. `Cargo.toml.orig`, `.cargo_vcs_info.json`, license files, tests, and all
unmodified release content are retained so the delta from the published crate remains auditable.

The dependency patches only replace five unmaintained dependencies in the resolved RuVector
feature graph:

- `number_prefix 0.4.0` is removed by applying hf-hub upstream commit
  `65fb0347d92c5569be93b334f547fc2f5c861ac6` to the exact `hf-hub 0.4.3` release and pinning
  `indicatif = 0.18.6` (which uses maintained `unit-prefix`).
- `paste 1.0.15` is replaced explicitly with `pastey = 0.2.3` in every direct owner. Both
  seven-package gemm release families intentionally keep direct `pastey` dependencies because
  their exported macros expand in the downstream crate's context. The `0.18.2` family is retained
  by the CUDA `candle-ug` metadata path even when it is not built for the host target, so it must
  be remediated as well: cargo-audit scans every package recorded in the lock.
- `serde_cbor 0.11.2` is replaced explicitly with Kanidm's maintained
  `serde_cbor_2 = 0.13.0` in pgrx. The pgrx source also carries upstream Rust-1.89 fix
  `2cb1b37ca0dca7f493896ff2a2e72f01ec921030`; no other pgrx behavior changes.
- `bincode 2.0.1` is replaced explicitly with `bincode_reloaded = 3.1.10` in the three exact
  RuVector owners. The format configuration remains `config::standard()` and every derive or
  serde call is otherwise unchanged.
- `bincode 1.3.3` is replaced explicitly with Servo's `fugue-bincode = 1.3.4` in hnsw_rs and
  ruvector-postgres. hnsw_rs's legacy format-v2 decoder is preserved; removing or refusing that
  branch would be a data-compatibility downgrade.

Two behavior-preserving unsafe-pointer hardenings are also applied to the release sources:

- pgrx's PostgreSQL list drains retain the existing `NonNull<pg_sys::List>` invariant until the
  raw pointer must be stored in `Drain`. A `List::Cons` allocation remains owned by its PostgreSQL
  memory context; `List::Nil` is represented explicitly as `None` before conversion back to null.
- ruvector-postgres's IVFFlat page writer reacquires the previous page header from the still-pinned,
  exclusively locked buffer instead of retaining a raw page pointer across loop iterations.

These changes make the lifetime proof visible to static analysis without changing allocation,
serialization, page layout, or list-drain behavior.

No replacement is hidden behind a Cargo package alias. The maintained package names appear in
both the patched manifests and Rust namespaces. The original names remain only in the published
`Cargo.toml.orig` baselines and this provenance document.

The root patch table uses version-qualified keys for the second gemm release family because Cargo
requires unique TOML keys. Those keys still declare `package = "gemm-*"`; they select two exact
versions of the same upstream packages and do not rename a replacement dependency.

## Exact release archives

The SHA-256 values are hashes of the downloaded `.crate` archives (and match the registry
checksums formerly recorded for these releases). Licenses are copied verbatim with each package.

| Package | Archive SHA-256 | License |
| --- | --- | --- |
| `hf-hub 0.4.3` | `629d8f3bbeda9d148036d6b0de0a3ab947abd08ce90626327fc3547a49d59d97` | Apache-2.0 |
| `gemm 0.18.2` | `ab96b703d31950f1aeddded248bc95543c9efc7ac9c4a21fda8703a83ee35451` | MIT |
| `gemm 0.19.0` | `aa0673db364b12263d103b68337a68fbecc541d6f6b61ba72fe438654709eacb` | MIT |
| `gemm-common 0.18.2` | `a352d4a69cbe938b9e2a9cb7a3a63b7e72f9349174a2752a558a8a563510d0f3` | MIT |
| `gemm-common 0.19.0` | `88027625910cc9b1085aaaa1c4bc46bb3a36aad323452b33c25b5e4e7c8e2a3e` | MIT |
| `gemm-c32 0.18.2` | `f6db9fd9f40421d00eea9dd0770045a5603b8d684654816637732463f4073847` | MIT |
| `gemm-c32 0.19.0` | `086936dbdcb99e37aad81d320f98f670e53c1e55a98bee70573e83f95beb128c` | MIT |
| `gemm-c64 0.18.2` | `dfcad8a3d35a43758330b635d02edad980c1e143dc2f21e6fd25f9e4eada8edf` | MIT |
| `gemm-c64 0.19.0` | `20c8aeeeec425959bda4d9827664029ba1501a90a0d1e6228e48bef741db3a3f` | MIT |
| `gemm-f16 0.18.2` | `cff95ae3259432f3c3410eaa919033cd03791d81cebd18018393dc147952e109` | MIT |
| `gemm-f16 0.19.0` | `e3df7a55202e6cd6739d82ae3399c8e0c7e1402859b30e4cb780e61525d9486e` | MIT |
| `gemm-f32 0.18.2` | `bc8d3d4385393304f407392f754cd2dc4b315d05063f62cf09f47b58de276864` | MIT |
| `gemm-f32 0.19.0` | `02e0b8c9da1fbec6e3e3ab2ce6bc259ef18eb5f6f0d3e4edf54b75f9fd41a81c` | MIT |
| `gemm-f64 0.18.2` | `35b2a4f76ce4b8b16eadc11ccf2e083252d8237c1b589558a49b0183545015bd` | MIT |
| `gemm-f64 0.19.0` | `056131e8f2a521bfab322f804ccd652520c79700d81209e9d9275bbdecaadc6a` | MIT |
| `pulp 0.22.3` | `046aa45b989642ec2e4717c8e72d677b13edd831a4d3b6cf37d9a3e54912496a` | MIT |
| `tokenizers 0.20.4` | `3b08cc37428a476fc9e20ac850132a513a2e1ce32b6a31addf2b74fa7033b905` | Apache-2.0 |
| `macro_rules_attribute 0.2.2` | `65049d7923698040cd0b1ddcced9b0eb14dd22c5f86ae59c3740eab64a676520` | Apache-2.0 OR MIT OR Zlib |
| `pgrx 0.12.9` | `227bf7e162ce710994306a97bc56bb3fe305f21120ab6692e2151c48416f5c0d` | MIT |
| `ruvector-core 2.3.0` | `ecaff2299e821f1f9aacd85f6fd16a16de6e7245b67843415d4f49f2f5f70084` | MIT |
| `ruvector-graph 2.2.3` | `ca6db1a3c778c441e13e0b4f90a64f6b3e0d1e8d9020021f1807f528b64a290e` | MIT |
| `ruvllm 2.3.0` | `e18a016fcc2eb3bffbcb3c2bb1d4d7420df820e11ba92a7a9f45d5c6779d423b` | MIT |
| `ruvector-postgres 2.0.5` | `052dadb088cb26e640833072416ad59a2b2437dbb534f7effe197e30261fe1d7` | MIT |
| `hnsw_rs 0.3.4` | `43a5258f079b97bf2e8311ff9579e903c899dcbac0d9a138d62e9a066778bd07` | MIT OR Apache-2.0 |

## Replacement release identities

| Replacement | Release source identity | MSRV |
| --- | --- | --- |
| `indicatif 0.18.6` | `e4d49d8ea6c68a80f7ee22904ee6c90322415a1d` | 1.85 |
| `pastey 0.2.3` | `c377d07c9d1a418e9922af59dc932736752a95cc` | 1.54 |
| `serde_cbor_2 0.13.0` | `431de98c556285c5f02c5a805cd59482bb9b23cc` | 1.81 |
| `bincode_reloaded 3.1.10` | `35d5d9f32d32e9f0694897f154ebdff6bdaf8fca` | 1.86 |
| `fugue-bincode 1.3.4` | `31e009b581653c8d657989da12771f39bf703d1c` | compatible with workspace 1.89 |

## Compatibility proof

`crates/engine/tests/ruvector_codec_compat.rs` contains bytes generated from the unmodified
published codecs and actual persisted RuVector shapes:

- bincode-2 direct derive values and the core HNSW state;
- graph `Node` and `Hyperedge` records;
- ruvllm `AdapterDataset` serde state;
- bincode-1 ruvector-postgres `SearchResult` and hnsw_rs format-v2 vector payloads;
- pgrx-style CBOR values, including a zero-copy borrowed string.

The maintained encoders must reproduce those bytes exactly and the maintained decoders must read
them. The old packages are not test dependencies, so the compatibility test cannot accidentally
reintroduce their advisories into `Cargo.lock`.

Run the focused proof with:

```bash
PGRX_PG_CONFIG_PATH=/home/flexnetos/.nix-profile/bin/pg_config \
  cargo +1.89.0 test -p envctl-engine --features ruvector-pg \
  --test ruvector_codec_compat
```
