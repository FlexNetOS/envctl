---
paths:
  - "**/prompt_hub/**"
  - "**/prompthub/**"
  - "**/prompthub-server/**"
---

# Rust conventions — prompthub / prompthub-server

- Rust toolchain is pinned per-repo (`rust-toolchain.toml`, prompt_hub pins 1.96.0). Do not float versions; do not add a second toolchain owner.
- snake_case files/modules/functions, PascalCase types, SCREAMING_SNAKE_CASE consts; tests are `#[cfg(test)] mod tests` beside the code or `tests/*.rs` integration files.
- `cargo fmt --all && cargo clippy --workspace -- -D warnings` must be clean before any commit.
- No new non-Rust source files: JS/Python drift is reverted or ported to Rust.
- Never `cargo publish`, never edit `Cargo.lock` by hand.
- Long builds/tests run as background bash (`run_in_background`), never blocking the main thread.
