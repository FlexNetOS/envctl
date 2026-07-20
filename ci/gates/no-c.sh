#!/usr/bin/env bash
# ci/gates/no-c.sh — fail-closed no-C / single-backend gate.
#
# Materialized from docs/ops/07-ci-supplychain.md §1.1 (keep the two in sync). Gate 3a ARMS
# automatically now that `crates/secrets-store-libsql` is a workspace member (OI-1 RESOLVED (a)).
# Upheld tenet: "no C *library* in the trust boundary" (no SQLite/OpenSSL/aws-lc). A build-time
# C toolchain (`cc`) is accepted — it is already mandatory via ring + blake3 in the engine, and the
# libSQL `remote` client adds only `libsql-sqlite3-parser`'s build-time `lemon.c` codegen (emits Rust;
# nothing C is linked). Run from the repo root: `bash ci/gates/no-c.sh`.
#
# HARDENING (audit wqj72spx0): every `cargo tree` is captured to a variable FIRST so a tree error
# fails the gate CLOSED (an inline `if cargo tree | grep` reads a failed/empty tree as "no C dep" and
# silently passes). Gate 4 reads the AUTHORITATIVE resolved graph from `cargo metadata` via python3,
# NOT `cargo tree -i`: the inverse tree errors "specification is ambiguous" (exit 101, empty stdout)
# the moment a crate has two versions — which, swallowed by `2>/dev/null || true`, was a FALSE PASS
# exactly when a second rustls/aws-lc would appear. `cargo metadata` is also immune to false-matching
# an optional/unresolved dependency *declaration* (e.g. rustls declares aws-lc-rs optional).
set -euo pipefail

fail() { echo "NO-C GATE FAIL: $*" >&2; exit 1; }

# --- Gate 1: the engine LIB is pure-Rust (always armed). DESIGN-NOTES R9, SERVER-MODE §79 ---
# --all-features so an optional/transitive C dep cannot hide behind a feature flag. Capture-first:
# a `cargo tree` failure aborts (fail-closed) instead of being misread as "no C dep".
ENGINE_TREE=$(cargo tree --locked -p envctl-secrets-engine --all-features --edges normal,build)
if grep -Eq 'libsql-ffi|sqlite3-sys|rusqlite|openssl-sys|aws-lc-sys|aws-lc-rs' <<<"$ENGINE_TREE"; then
  fail "C dependency linked into envctl-secrets-engine"
fi

# --- Gate 1.1: the GitHub RS256 signer stays on the exact fixed-width Ct/no-alloc graph. ---
# RUSTSEC-2023-0071 has no patched RustCrypto `rsa` release. The replacement is NOT allowed to
# drift back to rsa_heapless's allocating compatibility backend (which follows upstream security
# advisories), or silently feature-unify alloc/encoding/std/keygen/crypto-bigint into the signer.
# Query the resolved feature graph directly so a manifest declaration or a visually-clean tree
# cannot hide feature unification. Exact pins make this a deliberate security upgrade decision.
RSA_METADATA=$(cargo metadata --locked --format-version 1 --no-default-features \
  --features envctl-secrets-engine/inmem-store,envctl-secrets-engine/provider-github)
python3 -c '
import json,sys
m=json.load(sys.stdin)
packages={p["id"]:p for p in m["packages"]}
nodes={n["id"]:n for n in m["resolve"]["nodes"]}

def die(msg):
    sys.stderr.write("NO-C GATE FAIL: GitHub RS256 signer "+msg+"\n")
    sys.exit(1)

def exact_node(name, version):
    matches=[p for p in packages.values() if p["name"] == name and p["version"] == version]
    if len(matches) != 1:
        die(f"requires exact {name} {version}; resolved matches={len(matches)}")
    return matches[0], nodes[matches[0]["id"]]

if any(p["name"] == "rsa" for p in packages.values()):
    die("resolved the advisory-affected `rsa` package")

rsa,rsa_node=exact_node("rsa_heapless", "0.4.1")
fixed,fixed_node=exact_node("fixed-bigint", "0.5.2")
consts,consts_node=exact_node("const-num-traits", "0.2.0")
modmath,modmath_node=exact_node("modmath", "0.5.0")
cios,cios_node=exact_node("modmath-cios", "0.1.2")
pkcs1,pkcs1_node=exact_node("pkcs1", "0.8.0-rc.4")
pkcs8,pkcs8_node=exact_node("pkcs8", "0.11.0")
sha2,sha2_node=exact_node("sha2", "0.11.0")

expected={
    "rsa_heapless": ({"modmath"}, set(rsa_node["features"])),
    "fixed-bigint": ({"cios","use-unsafe","zeroize"}, set(fixed_node["features"])),
    "const-num-traits": ({"ct"}, set(consts_node["features"])),
    "modmath": ({"zeroize"}, set(modmath_node["features"])),
    "modmath-cios": (set(), set(cios_node["features"])),
    "pkcs1": (set(), set(pkcs1_node["features"])),
    "pkcs8": (set(), set(pkcs8_node["features"])),
    "sha2@0.11": ({"oid"}, set(sha2_node["features"])),
}
for name,(want,got) in expected.items():
    if got != want:
        die(f"feature drift for {name}: expected {sorted(want)}, got {sorted(got)}")

# Traverse the arithmetic backend itself and reject native-link declarations/tooling or
# crypto-bigint. Pure-Rust proc-macro crates may legitimately have a Rust `build.rs`, so a generic
# `custom-build` target is not itself evidence of C; the Cargo `links` field plus native build-tool
# packages are the fail-closed native boundary. The outer engine still contains the sanctioned ring
# provider; this scoped traversal proves the newly-added RSA arithmetic stack is Rust-only.
seen=set()
stack=[rsa["id"], fixed["id"], modmath["id"], cios["id"]]
while stack:
    package_id=stack.pop()
    if package_id in seen:
        continue
    seen.add(package_id)
    package=packages[package_id]
    if package["name"] == "crypto-bigint":
        die("re-enabled the allocating crypto-bigint backend")
    if package.get("links"):
        die("introduced native linkage via {} {} (links={})".format(package["name"], package["version"], package["links"]))
    if package["name"] in {"cc","cmake","pkg-config","vcpkg","bindgen"}:
        die("introduced native build tooling via {} {}".format(package["name"], package["version"]))
    stack.extend(dep["pkg"] for dep in nodes[package_id]["deps"])

print("GitHub RS256 graph clean: rsa_heapless=0.4.1[modmath], fixed-bigint=0.5.2[cios,use-unsafe,zeroize], modmath=0.5.0[zeroize], modmath-cios=0.1.2[], no alloc/crypto-bigint/native build")
' <<<"$RSA_METADATA"

# --- Gate 1.5: the ENGINE crate stays C-free after adopting loop_lib (meta substrate). ---
# The engine's hook runner delegates Command construction to `loop_lib::build_command`. loop_lib is
# pure-Rust (anyhow/rayon/serde/serde_json/colored/indicatif/is-terminal — zero C library). This locks
# in that the meta-wiring adoption introduced no banned C dep into the engine. Capture-first.
# NOTE: this gate (and the whole-graph Gate 4) now require the meta tree to be present, because
# envctl-engine path-deps `../../../loop_lib`; envctl is a meta-tree-resident crate by design.
ENGINE_LIB_TREE=$(cargo tree --locked -p envctl-engine --all-features --edges normal,build)
if grep -Eq 'libsql-ffi|sqlite3-sys|rusqlite|openssl-sys|aws-lc-sys|aws-lc-rs|mimalloc|libmimalloc-sys' <<<"$ENGINE_LIB_TREE"; then
  fail "C dependency linked into envctl-engine (loop_lib adoption must stay pure-Rust)"
fi

# --- Gate 2: proto + cli stay C-free (SERVER-MODE §81) ---
for crate in envctl-secrets-proto envctl-secretctl; do
  CRATE_TREE=$(cargo tree --locked -p "$crate" --all-features --edges normal,build)
  if grep -Eq 'libsql-ffi|sqlite3-sys|openssl-sys|aws-lc-sys|aws-lc-rs' <<<"$CRATE_TREE"; then
    fail "C dependency linked into $crate"
  fi
done

# --- Gate 3: store-crate scoped waiver (auto-arms when the crate exists). SERVER-MODE §80 ---
if cargo metadata --locked --no-deps --format-version 1 | grep -q '"name":"envctl-secrets-store-libsql"'; then
  # 3a: the SHIPPING wiring (pure-Rust `remote` client) MUST link no C SQLite. Capture-first.
  STORE_TREE=$(cargo tree --locked -p envctl-secrets-store-libsql --no-default-features --features remote)
  if grep -Eq 'libsql-ffi|libsql-sys|sqlite3-sys' <<<"$STORE_TREE"; then
    fail "remote-client build of store crate links a C SQLite (libsql-ffi/libsql-sys/sqlite3-sys)"
  fi
  # 3b: no-op note (honest). The `embedded` feature (a future risk-accepted in-process C-SQLite
  #     fallback) is an UNIMPLEMENTED placeholder that pulls no libsql feature, so there is nothing to
  #     scope yet. If it is ever implemented, add an assertion here that libsql-ffi is reachable ONLY
  #     via this crate. Workspace-wide absence of libsql-ffi is already proven by Gate 4 below.
  echo "note: store-crate 'embedded' (in-process C-SQLite) is an unbuilt placeholder; remote-only ships"
fi

# --- Gate 3.5: agent-env crate (kasetto absorption, Epic C) stays C-free — incl. NO mimalloc allocator. ---
# Auto-arms when crates/agent-env exists. kasetto upstream links the mimalloc C allocator; the absorb
# MUST drop mimalloc/libmimalloc-sys (see .claude/skills/rust-feature-impl/references/kasetto-absorption.md).
if cargo metadata --locked --no-deps --format-version 1 | grep -q '"name":"envctl-agent-env"'; then
  AGENTENV_TREE=$(cargo tree --locked -p envctl-agent-env --all-features --edges normal,build)
  if grep -Eq 'libsql-ffi|sqlite3-sys|rusqlite|openssl-sys|aws-lc-sys|aws-lc-rs|mimalloc|libmimalloc-sys' <<<"$AGENTENV_TREE"; then
    fail "C dependency (incl. mimalloc allocator) linked into envctl-agent-env — drop it per the no-downgrade absorption playbook"
  fi
fi

# --- Gate 4: exactly one ring-only rustls; zero aws-lc/openssl/C-SQLite ANYWHERE. DESIGN-NOTES R7, CF-2 ---
# Authoritative resolved graph (`cargo metadata` .resolve.nodes), parsed with python3.
cargo metadata --locked --format-version 1 | python3 -c '
import json,sys
m=json.load(sys.stdin)
idmap={p["id"]:(p["name"],p["version"]) for p in m["packages"]}
resolved={}
for node in m["resolve"]["nodes"]:
    name,ver=idmap[node["id"]]
    resolved.setdefault(name,set()).add(ver)
def die(msg):
    sys.stderr.write("NO-C GATE FAIL: "+msg+"\n"); sys.exit(1)
banned=["aws-lc-sys","aws-lc-rs","openssl-sys","libsql-ffi","libsql-sys","sqlite3-sys","rusqlite"]
present=[c for c in banned if resolved.get(c)]
if present:
    die("forbidden C crate(s) resolved into the graph: "+", ".join(c+" "+str(sorted(resolved[c])) for c in present))
rv=sorted(resolved.get("rustls",[]))
if len(rv)>1:
    die("more than one rustls version in the graph: "+str(rv))
if not resolved.get("ring"):
    die("ring backend not present — rustls crypto-provider pin broke")
print("resolved graph clean: rustls="+(str(rv) if rv else "none")+" on ring="+str(sorted(resolved["ring"]))+"; zero aws-lc/openssl/C-SQLite")
'

echo "NO-C GATE PASS"
