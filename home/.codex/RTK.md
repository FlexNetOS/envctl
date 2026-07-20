# RTK policy pointer

The sole Codex RTK policy is `/home/flexnetos/.codex/AGENTS.rtk.md`. It requires
the exact profile-owned `/home/flexnetos/.nix-profile/bin/rtk` frontdoor and
`rtk proxy --` for unfiltered native output. This compatibility file is not a
second policy surface and is intentionally not imported by `AGENTS.md`.
