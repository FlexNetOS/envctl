# Strict profile runtime cutover

## Authority

The canonical `FlexNetOS/yazelix` repository builds the sole installed
frontdoor at `/home/flexnetos/.nix-profile`. Envctl validates and projects that
contract; it does not install a parallel agent runtime or PATH layer.

Mutable Codex and Claude state lives only under
`/home/flexnetos/meta/var/lib/{codex,claude}`. Reviewed configuration
inputs are installed below the profile `share/yazelix/agent_configs` tree and
materialized by profile-owned commands.

## Retired projection receipt

The former envctl-owned home projections remain recoverable from immutable Git
objects and are not active authority:

| Record | Object |
|---|---|
| Source commit | `3c31f3cc1a22ba5704e06f33c32fb855b41de4b7` |
| Codex projection tree (131 files) | `75ed55804d369345c2c144fccc073e8db27bdfde` |
| Claude projection tree (39 files) | `a9ec88e6226ed175cf6e186827b5826f9ed28cc9` |

Those objects are recovery evidence only. Restoring them as installed input,
state, cache, PATH, launcher, or compatibility ownership is prohibited.

## Reviewed installed inputs

| Input | SHA-256 |
|---|---|
| Codex configuration | `e01f860d3d3a6bbfbf1aeff1bd2920d71eb77cf9e0d40d87797056c8e9ae4c53` |
| Codex operating rules | `ebf611b228f3f3177ff9c477b55653b61ecc375e28a6724ce4f0a3f9afd34ecc` |
| Claude settings | `fee65c24a55d8ddbe00583b2d3e3b6567c589d9a7a2ddb89dc788b827d221c69` |
| Claude operating contract | `81f641f567909e245c48b8edcbac474decb2db8df93e76138ce82bb1ddc4c2fb` |
| Claude RTK contract | `c4102c99560467a3911a46ac866008a186b306187efcdbaeec52e6a3c585bf39` |

The final installed proof must be generated from a clean checkout of merged
Yazelix `origin/main`; build outputs from feature branches are verification
only and are never final cutover authority.
