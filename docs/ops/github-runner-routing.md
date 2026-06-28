# GitHub runner routing policy

`envctl` uses both GitHub-hosted runners and the FlexNetOS organization self-hosted runner. The goal is to keep the protected status contexts stable while avoiding the single local runner becoming a serial bottleneck.

## Required context routing

The `develop` branch protection currently requires these status contexts:

- `rustfmt`
- `clippy`
- `MSRV (Rust 1.88)`
- `cargo audit`
- `test`
- `gates`

Those job names are a policy contract. Do not rename them without updating branch protection in the same change.

Current routing:

| Job/context | Runner policy | Reason |
| --- | --- | --- |
| `rustfmt` | `ubuntu-latest` | Fast static fan-out; should not wait behind local heavy tests. |
| `clippy` | `ubuntu-latest` | Compile-heavy, but parallel hosted capacity is faster than serializing all checks on one local runner. |
| `MSRV (Rust 1.88)` | `ubuntu-latest` | Independent compatibility gate; benefits from hosted fan-out. |
| `cargo audit` | `ubuntu-latest` | Network/tooling gate; keep it off the single local queue. |
| `gates` | `ubuntu-latest` | Policy/invariant gate; also validates this runner-routing policy. |
| `test` | trusted refs: `[self-hosted, linux, x64, local, flexnetos]`; fork PRs: `ubuntu-latest` | The workspace test job is the cache-heavy local workload. Fork PRs must never execute untrusted code on the local host. |

`sync-master` is a trusted maintenance workflow and remains on `[self-hosted, linux, x64, local, flexnetos]`.

## Queue policy

CI cancels stale non-`develop` runs with workflow concurrency. Protected `develop` runs are never cancelled because `sync-master` consumes their completed state before fast-forwarding `master`.

## Regression guard

`ci/gates/runner-routing.sh` fails closed if the workflow drifts from this split. The `gates` job runs that check first so a PR cannot silently route every required status context back onto the single local runner.
