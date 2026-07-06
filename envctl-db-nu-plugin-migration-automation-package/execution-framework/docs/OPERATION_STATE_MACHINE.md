# envctl operation state machine

Generated at: `2026-07-04T23:09:12+00:00`
Status: `passed`

## Canonical states

| state | persisted operation status | terminal |
|---|---|---|
| `planned` | `queued` | no |
| `runnable` | `ready` | no |
| `approved` | `ready` | no |
| `running` | `running` | no |
| `blocked` | `blocked` | no |
| `failed` | `failed` | no |
| `completed` | `succeeded` | yes |
| `rolled_back` | `cancelled` | yes |

## Transitions

| from | allowed targets |
|---|---|
| `planned` | `approved`, `blocked`, `failed`, `runnable` |
| `runnable` | `approved`, `blocked`, `failed`, `running` |
| `approved` | `blocked`, `failed`, `running` |
| `running` | `blocked`, `completed`, `failed`, `rolled_back` |
| `blocked` | `approved`, `failed`, `rolled_back`, `runnable` |
| `failed` | `rolled_back` |
| `completed` | `rolled_back` |
| `rolled_back` | _none_ |

## Guards

- `R3` through `R5` operations must enter `approved` before `running`.
- `completed` and `rolled_back` are terminal for dispatch; `completed` can still be compensated by a rollback transition.
- The persisted SQL enum remains the existing operation schema enum; canonical state is bridged through the state machine mapping.

## Verification

- Required canonical states covered: `8`
- Transition path events exercised: `12`
- Illegal transitions rejected: `3`
- SQLite bridge rows inserted: `8`
