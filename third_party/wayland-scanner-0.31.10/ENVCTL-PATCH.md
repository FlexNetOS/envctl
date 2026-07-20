# envctl security patch provenance

This directory is the exact crates.io source release of `wayland-scanner`
0.31.10, with the minimal upstream-compatible patch needed to consume
`quick-xml` 0.41.

- crates.io archive SHA-256:
  `9c324a910fd86ebdc364a3e61ec1f11737d3b1d6c273c0239ee8ff4bc0d24b4a`
- upstream repository: <https://github.com/smithay/wayland-rs>
- upstream compatibility commit:
  `ec2d932855593d48aa83c76820f3efbcfea86d39` (`xml_content` to
  `xml10_content`)
- upstream security dependency commit:
  `d07c4f91f28b42e5a485823ffd9d8d5a210b1053` (`quick-xml` 0.41)
- retained upstream license: MIT (`LICENSE.txt`)

No other upstream code change is included. Keeping the published 0.31.10
source avoids importing the unrelated unreleased and breaking changes on the
wayland-rs development branch while removing RUSTSEC-2026-0194 and
RUSTSEC-2026-0195 from envctl's resolved graph.

The only packaging-only addition is an empty Cargo workspace boundary, which
makes this crate's own locked tests runnable when envctl itself is nested in a
meta-managed worktree set. It does not change the crate's compiled API or
dependency behavior.
