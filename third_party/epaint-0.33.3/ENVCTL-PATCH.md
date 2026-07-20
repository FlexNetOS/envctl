# envctl `epaint` 0.33.3 security backport

This directory starts from the exact `epaint` 0.33.3 crate published on
crates.io. The source archive has SHA-256:

```text
009d0dd3c2163823a0abdb899451ecbc78798dec545ee91b43aff1fa790bab62
```

The archive's `.cargo_vcs_info.json` identifies upstream egui commit
`44cdd653e2317d300fb8a6c9c36b03f23991e803` and the path
`crates/epaint`.

Only the font parser/rasterizer migration from these upstream commits is
backported:

- `609dd2d28edfadd544f53cec39b38564eb4fcb75` — replace `ab_glyph` with
  Skrifa and `vello_cpu`, and enable hinting.
- `6277a310b93f2f07834e920baabe43409334c973` — disable Skrifa's unused
  `traversal` default feature while retaining `std` and `autohint_shaping`.

The source changes were transplanted onto the published 0.33.3 crate, rather
than upgrading the whole egui family. A compatibility shim deliberately keeps
the stock 0.33.3 `FontTweak`, `Fonts`, `FontsView`, `FontsImpl`,
`TextureAtlas`, and glyph metric public interfaces. The newer `TextOptions`
is internal to the migrated implementation; stock constructors select the
upstream migration's default of enabled font hinting. This lets unmodified
egui/eframe 0.33.3 consume the patched crate.

The published crate omitted license files from its package archive even
though its manifest declares `MIT OR Apache-2.0`. Exact `LICENSE-MIT` and
`LICENSE-APACHE` files were copied from the upstream 0.33.3 tag. The benchmark
keeps the upstream source but drops its `mimalloc` development-only allocator
so this independently testable vendor boundary cannot resolve a C allocator.
Upstream's `font-test-data` development dependency is retained for font-format
regression tests. Four stock `#[expect]` lint annotations are omitted because
Clippy 1.89 reports the expectations as unfulfilled; this is a lint-only MSRV
compatibility adjustment.

This is a source backport, not a forked public API or a version upgrade. Any
future update must re-verify the archive checksum, both commit IDs, license
texts, the 0.33 compatibility surface, the locked feature tree, MSRV, and the
workspace no-C and advisory gates.
