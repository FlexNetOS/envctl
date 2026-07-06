# Applied Upgrades

Upgrade-only policy: source package files were preserved. The execution framework was added as an additive layer.

## UPG-001 → GAP-001

- Files added: `execution-framework/README.md`, `execution-framework/docs/**`, `execution-framework/scripts/**`, `execution-framework/schemas/**`, `execution-framework/templates/**`, `execution-framework/generated/**`, `execution-framework/proof_records/**`, `execution-framework/state/**`
- Files modified: None
- Why needed: Create full execution framework layer.
- Downstream consumer: Codex, background helpers, envctl database automation, nu_plugin control surface.
- Verification: python3 scripts/verify_history_and_completeness.py
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

## UPG-002 → GAP-002

- Files added: `execution-framework/generated/task_graph.csv`, `execution-framework/generated/task_graph.normalized.json`, `execution-framework/generated/task_graph.index.json`
- Files modified: None
- Why needed: Create task graph with required columns and task coverage.
- Downstream consumer: /goal loop and packet generator.
- Verification: python3 scripts/validate_task_graph.py generated/task_graph.csv
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

## UPG-003 → GAP-003

- Files added: `execution-framework/generated/execution_manifest.json`, `execution-framework/generated/execution_packets/*.json`
- Files modified: None
- Why needed: Create executable JSON packet pipeline.
- Downstream consumer: Background helpers and Codex CLI.
- Verification: python3 scripts/task_graph_to_packets.py generated/task_graph.csv
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

## UPG-004 → GAP-004

- Files added: `execution-framework/proof_records/proof_ledger.jsonl`, `execution-framework/proof_records/*.proof.json`, `execution-framework/schemas/proof_record.schema.json`
- Files modified: None
- Why needed: Create proof substrate.
- Downstream consumer: Verification and repeatable execution ledger.
- Verification: python3 scripts/verify_history_and_completeness.py
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

## UPG-005 → GAP-005

- Files added: `execution-framework/docs/MULTI_AGENT_COLUMNS.md`, `execution-framework/templates/AGENT_LANE_TEMPLATE.json`, `execution-framework/schemas/agent_lane.schema.json`
- Files modified: None
- Why needed: Create multi-agent lane control contract.
- Downstream consumer: Spark helpers, Codex background shells, nu_plugin status UI.
- Verification: schema parse + final verification
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

## UPG-006 → GAP-006

- Files added: `execution-framework/scripts/goal_loop.py`, `execution-framework/state/goal_loop_state.json`, `execution-framework/generated/status_report.json`
- Files modified: None
- Why needed: Create /goal loop execution control.
- Downstream consumer: envctl database automation and agent orchestrators.
- Verification: python3 scripts/goal_loop.py generated/task_graph.csv
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

## UPG-007 → GAP-007

- Files added: `history/pre_execution_framework_manifest.json`, `history/README.md`
- Files modified: None
- Why needed: Record prior source package state.
- Downstream consumer: Rollback and history verification.
- Verification: python3 scripts/verify_history_and_completeness.py
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

## UPG-008 → GAP-008

- Files added: `execution-templates/README.md`, `execution-templates/TASK_GRAPH_TEMPLATE.csv`, `execution-templates/AGENT_LANE_TEMPLATE.json`, `execution-templates/two_repo_parallel_goal.yaml`
- Files modified: None
- Why needed: Add top-level execution templates for operator file organization.
- Downstream consumer: Google Drive folder organization and Codex handoff.
- Verification: test -d execution-templates
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

## UPG-009 → GAP-009

- Files added: `execution-framework/generated/final_verification_report.json`, `execution-framework/docs/FINAL_VERIFICATION.md`
- Files modified: None
- Why needed: Document Drive write/history external blocker while producing uploadable upgraded archive.
- Downstream consumer: Human operator / Drive upload workflow.
- Verification: final report external_blockers
- Rollback: Remove added files or restore from history/pre_execution_framework_manifest.json; no source package files were overwritten.

# Additive File Ledger

Every added file currently present in `execution-framework/`, `execution-templates/`, `history/`, plus the preserved attached prompt copy is listed below.

| Path | Size | SHA-256 |
|---|---:|---|
| `execution-framework/README.md` | 1157 | `040cdd8b710e242b31814d4c9729ff1950f38ed65279ed4715bc3027d70e9355` |
| `execution-framework/docs/APPLIED_UPGRADES.md` | 5079 | `631a7ef645588e21422f1d0054f3eb3bf2d0fe948e2b6fe6ad74798336ad46d5` |
| `execution-framework/docs/EXECUTION_FRAMEWORK_INSTALL.md` | 815 | `64d13952bf890966681f1424d7b2d549273124706615746e0d008367c701fe52` |
| `execution-framework/docs/FINAL_VERIFICATION.md` | 1393 | `829535c2b2d7bf9bc0bb01ddfcf4243375a1419ea2303f01ae42044a80267bdd` |
| `execution-framework/docs/GAP_ANALYSIS.md` | 4239 | `a92e49ab23b14e3991707132676549b1de583f84294576a83ce14b1369eae829` |
| `execution-framework/docs/GOAL_LOOP_PROTOCOL.md` | 820 | `c228d3d63617424676cf152a5131510ab22485d1d6ffc356c969d1d9b6ac774d` |
| `execution-framework/docs/MULTI_AGENT_COLUMNS.md` | 1123 | `23877878f15fa77ae194b2300d04a3101bcec51d5ebd69f5a5dd332f361c2e04` |
| `execution-framework/examples/two_repo_parallel_goal.yaml` | 889 | `286b45dd7eee83deb37a6d3a164d87eb3e5872beee46584be4de4fc2e2803234` |
| `execution-framework/generated/execution_manifest.json` | 27668 | `fec37aba5dc59614841cd47b7dd0314d3ecbbcada18fbe2ddcd14251203c31d5` |
| `execution-framework/generated/execution_packets/ART-100_SYSTEM_INVENTORY.json` | 2426 | `62f37b977229dfc7fa52727037ac6f56a701d46b61906a877ac1c5fa4f00d3e9` |
| `execution-framework/generated/execution_packets/ART-101_DIRECTORY_TREE.json` | 2431 | `511d7aa890683d42248049c8f073956a91b03949e2cb4f2964f3bfa4699bed71` |
| `execution-framework/generated/execution_packets/ART-102_REPOSITORY_MAP.json` | 2365 | `f77399cf3350989edcab2a0e7b9efdb1ba3e7b7d7d14165c9325f1e975ed6d95` |
| `execution-framework/generated/execution_packets/ART-103_SERVICE_DEP_GRAPH.json` | 2435 | `e06c2d62462ef3f3f5b4eadf990266aec2ecf7405957fb33c91a5f655238deb2` |
| `execution-framework/generated/execution_packets/ART-104_TOOLCHAIN_TREE.json` | 2411 | `189fd6cbb7458e7122380cb96c85e6856024303e39253d80f3a444d01d7158fe` |
| `execution-framework/generated/execution_packets/ART-105_PACKAGE_LIB_GRAPH.json` | 2452 | `aea984cfa883b55b5a3bedb06a999682edf878a8a305b6a298850d9ff5ffc29c` |
| `execution-framework/generated/execution_packets/ART-106_RUNTIME_DEP_MAP.json` | 2403 | `a9dc7a5b2b1139c02c6003bd9271f92f16715f6a44a9cfcca5a01f400c94412a` |
| `execution-framework/generated/execution_packets/ART-107_DATA_FLOW_GRAPH.json` | 2389 | `e28613407ae51063fcab3456e6e1006797e6f5a366935c899644c903f013b747` |
| `execution-framework/generated/execution_packets/ART-108_DB_SCHEMA_MAP.json` | 2394 | `638372ce6367555f699806b618e5f1192275af904207ae4dde84f8a4bc49dda1` |
| `execution-framework/generated/execution_packets/ART-109_DATA_LINEAGE.json` | 2373 | `bec79ea22783b482e35b7a6a0be530885134287370190866d86c14eba16e1c77` |
| `execution-framework/generated/execution_packets/ART-110_API_CATALOG.json` | 2368 | `86651486ec0228d9bdb0b9c1c340efc436a39b01b4f4724b2aec96e416b58c9f` |
| `execution-framework/generated/execution_packets/ART-111_EVENT_MAP.json` | 2375 | `905f4b67767c894a76dee84ae37901e0f80cbe6f9bcff953e609334a6cf5d55e` |
| `execution-framework/generated/execution_packets/ART-112_CODE_OWNERSHIP.json` | 2371 | `a648f8057b7e5f19bee6b12359ca7d437e3233184c3c73edc8bb7ce09e2a1e26` |
| `execution-framework/generated/execution_packets/ART-113_DEBUG_CODE_MAP.json` | 2424 | `e86b211833bb25e220a9b3bee253fc3c43b5fafecec55a6961133666eb538bd0` |
| `execution-framework/generated/execution_packets/ART-114_ENV_CONFIG_MATRIX.json` | 2380 | `134d879fb8e79392e6f9a3f64fac6560abbd28294c444a4fc30a107a82e7e0e0` |
| `execution-framework/generated/execution_packets/ART-115_CONFIG_INVENTORY.json` | 2413 | `2fa423b871370bf886824694dfd0264316ba51dd8753c213d34c34d74dd65c28` |
| `execution-framework/generated/execution_packets/ART-116_INFRA_TOPOLOGY.json` | 2425 | `56bcec3f0c84c09c80af460c08f3098a6b1bf89b4dca64dd6a1915485440a256` |
| `execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json` | 2378 | `21dfab2ae919eabfc8e5fda34ae8427c2597d9a9960b9e33290e94ff07f5a855` |
| `execution-framework/generated/execution_packets/ART-118_OBSERVABILITY.json` | 2381 | `a002205247b4902156ee3e7f31579494c60aa0a3f9b76d5a9b4e192105906c41` |
| `execution-framework/generated/execution_packets/ART-119_BUSINESS_PROCESS.json` | 2390 | `ef2c95d7a9d303387a0e742c98ba060d69ec7a98ebdcfc145d5a5349d4dee4e6` |
| `execution-framework/generated/execution_packets/ART-120_WAVE_PLAN.json` | 2332 | `4ef3340f574341eb779ebcee2c8a1a6e097f690a08966d01baf06dcb0e54847c` |
| `execution-framework/generated/execution_packets/ART-121_CUTOVER.json` | 2338 | `4d42be8fcfe39094f63e4fb734078cbe3c0e959575db2e82c36041d3c05fbf9a` |
| `execution-framework/generated/execution_packets/ART-122_ROLLBACK.json` | 2343 | `8b5e6c26dfc33ed16433946beb23b712967aad185aedb5df1526664ad5e1b2c5` |
| `execution-framework/generated/execution_packets/ART-123_VALIDATION_RECONCILIATION.json` | 2524 | `ea0724df016bfad5fd158daed790a78cb25286f7279f0a4b3cb0469d0510707f` |
| `execution-framework/generated/execution_packets/ART-124_TEST_COVERAGE.json` | 2429 | `96dd9d49d096c5eb520f5489aa9896f36002c12f26fefb5de77cde03fabd73eb` |
| `execution-framework/generated/execution_packets/ART-125_RISK_REGISTER.json` | 2360 | `738247430b0689f1f89d66d3ebbd30bc1de17f676070d1c52d8f9f99fbc5a15f` |
| `execution-framework/generated/execution_packets/ART-126_DECISION_LOG.json` | 2349 | `97f3499018385e90fc4564864a8594c396618e521f1fc80b82366e8c69a84825` |
| `execution-framework/generated/execution_packets/ART-127_BLAST_RADIUS.json` | 2370 | `72e7c0d95ac8426e42456c43c1d43b732c988c26e74825059d4d09e0b9b39855` |
| `execution-framework/generated/execution_packets/ART-128_READINESS_SCORECARD.json` | 2414 | `a08daad6f6dd1de52f9ffc0aa9eb43a772a9180e9f2b2237c4646cec2f6e5a08` |
| `execution-framework/generated/execution_packets/ART-129_BUSINESS_CAPABILITY.json` | 2417 | `496cbb2aca8e7746844b53e329dff294b9deda6392b4b44d79c7b1e745514014` |
| `execution-framework/generated/execution_packets/ART-130_SHADOW_TRAFFIC.json` | 2396 | `fda3d9a6e2b3d59b8d441228db802dbd0e375b5d111521b82251e7f56afc0e97` |
| `execution-framework/generated/execution_packets/ART-131_GOLDEN_DATASET.json` | 2353 | `74649bea863657f36760ce840033599844ef1367fa3ae0a1a4133a9b0f6abc2b` |
| `execution-framework/generated/execution_packets/ART-132_PARITY_DASHBOARD.json` | 2424 | `d6a5033d8c4fd2cbd311d6cc3188c1feb90ca5021eed9c75a52230fe903d642e` |
| `execution-framework/generated/execution_packets/ART-133_DEPRECATION_MAP.json` | 2379 | `6323c258c26a3dfd57442ddddca1772743a107055591681e08189fc959c322a0` |
| `execution-framework/generated/execution_packets/ART-134_EXCEPTION_INVENTORY.json` | 2418 | `cc3b8975aafc0f53127ce6a16c8977f85a6c639dedb1184b40250abfa9626734` |
| `execution-framework/generated/execution_packets/ART-135_RACI.json` | 2318 | `132514b197d3273d7948320bd620c2b994cfc57da5d112237137a141c75a6837` |
| `execution-framework/generated/execution_packets/ART-136_TECH_DEBT_LEDGER.json` | 2385 | `e19c7dbd664dd575a6d2e182b608f8360fe0dd1c3dbb4f59af8a3ee01964e4c6` |
| `execution-framework/generated/execution_packets/EF-001_SCAN_PACKAGE.json` | 2019 | `9f4aa76b04090b7d2d3ccef4402fc3adb4003b1aedec29dcda85e30adfcd1733` |
| `execution-framework/generated/execution_packets/EF-002_GAP_REPORT.json` | 1932 | `00235c947d9f11aadbf0aa00ffbd7a056014a26add6028bc5722e6701bd7548d` |
| `execution-framework/generated/execution_packets/EF-003_APPLY_FRAMEWORK.json` | 2196 | `676b7b85a65d8cfd051c4909fd75a12177a565e3a89c4cdf99cffe708a14bdfc` |
| `execution-framework/generated/execution_packets/EF-004_GENERATE_TASK_GRAPH.json` | 2178 | `690798f4223c042823c8943ff042f818c07f33d864306ea88cc5440ca9d61e6c` |
| `execution-framework/generated/execution_packets/EF-005_VALIDATE_TASK_GRAPH.json` | 2067 | `d1eeb92a0cf69598324c3e44f54a943a025f115561b461e751e21a9e3160440d` |
| `execution-framework/generated/execution_packets/EF-006_TASK_GRAPH_TO_PACKETS.json` | 2127 | `3b0718b134bf529440dd68dddb425801c8f12a291366662debbfce7363038c32` |
| `execution-framework/generated/execution_packets/EF-007_GOAL_LOOP.json` | 2139 | `eccd2335f50f72d6ddfb3da78d5e322ba3a403cb1bbf1c0e09e9665d8048dee1` |
| `execution-framework/generated/execution_packets/EF-008_VERIFY_HISTORY_COMPLETENESS.json` | 2412 | `8291ef633fb67a22555690005c3355b184f81605aad4fcbb8243fa034d61e415` |
| `execution-framework/generated/execution_packets/REL-400_PACKAGE_ARCHIVE.json` | 2184 | `b489737cb0f89f5e5c4df46f82f47d0579031a1da57f50abb15effb5026bbd2e` |
| `execution-framework/generated/execution_packets/REL-401_HANDOFF.json` | 2204 | `7b5ad6ac0054473a56ff2c857910980a88b941a9cd212dd0e7686b64bd73c4d9` |
| `execution-framework/generated/execution_packets/REQ-010_CONTRACT_LOCK.json` | 2267 | `7bf2b265dbf19ca5ecd40e04f618bb2d33cd7ab9bbe9ac7318973deac9e40add` |
| `execution-framework/generated/execution_packets/REQ-020_ENVCTL_DB_SCHEMA.json` | 2339 | `94b14b9475b327ee68d3ab1d9314c177469a3a2ead7263f47f2f4a47b423ed36` |
| `execution-framework/generated/execution_packets/REQ-021_ENVCTL_TARGET_REGISTRY.json` | 2257 | `2c818f5b0703fdf0042405a68a8f7749ba507367c000a476ddafc7a65aec04f7` |
| `execution-framework/generated/execution_packets/REQ-022_ENVCTL_RUN_LEDGER.json` | 2218 | `7160c68aef71df586737737c865bbe0ddb0912179985ef310ac27c89dd4a353b` |
| `execution-framework/generated/execution_packets/REQ-023_ENVCTL_OPERATION_STATE.json` | 2284 | `a0b83889ed24e43d75bee9d2314a6e9fed5fc0c92a02ff4139b19aec7b4b4e02` |
| `execution-framework/generated/execution_packets/REQ-024_ENVCTL_ARTIFACT_REGISTRY.json` | 2288 | `43f044b4546f0d85259ea0914c739ad38ef855d7b8a94aca932e8e4c7f7a6192` |
| `execution-framework/generated/execution_packets/REQ-025_ENVCTL_VALIDATION_EVIDENCE.json` | 2253 | `d11c1e83bca2427522d59016d629096f883a7d91d3b7cee13528851db1ddec2d` |
| `execution-framework/generated/execution_packets/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.json` | 2267 | `bbc5e9d6b69d23a326b5f906f9454adcf3155d384661bdcb881d0531b58328dd` |
| `execution-framework/generated/execution_packets/REQ-027_ENVCTL_REPLAY_ENGINE.json` | 2384 | `cb481605a002b066f8805e685477c29f297dbca59a76324544271f8c0d147876` |
| `execution-framework/generated/execution_packets/REQ-028_ENVCTL_AGENT_CONTROL_API.json` | 2334 | `21f7e006037950242590c26ee8cbc54f5cfd3148d7e21ade6111f94908f897ad` |
| `execution-framework/generated/execution_packets/REQ-030_PLUGIN_PROTOCOL_MANIFEST.json` | 2281 | `85098886972ce0930636706d15c9c9fbdca58d1a49db02585b62e004f73457c5` |
| `execution-framework/generated/execution_packets/REQ-031_PLUGIN_COMMAND_SURFACE.json` | 2290 | `4ef4bba7ee73f1969fc2d8782fa39bd53b97f6582b2d3aa4e1b0feedd1b9efc5` |
| `execution-framework/generated/execution_packets/REQ-032_PLUGIN_LIVE_VISUALS.json` | 2253 | `3449a8e69cf0040fee1490b97e891df79805469ec005bc0ed2e85fb200fd8b3d` |
| `execution-framework/generated/execution_packets/REQ-033_PLUGIN_HUMAN_APPROVAL.json` | 2275 | `5d5d5112e311ba4e92989553a8df0cac8269411c38159347e5a541a354105db1` |
| `execution-framework/generated/execution_packets/REQ-034_PLUGIN_STATUS_STREAMS.json` | 2253 | `6816b1d9b53bf9ebeca861705267d830717190de2c6068248fb37603f1871823` |
| `execution-framework/generated/execution_packets/REQ-040_SHARED_PROTOCOL_SCHEMAS.json` | 2289 | `9fa633ef8a441f87a182eaa5b32fccaa761d129362388b06d440c8150defdd69` |
| `execution-framework/generated/execution_packets/REQ-041_TWO_REPO_INTEGRATION.json` | 2554 | `9d7ccf217e95771e2e162d21d8911e0a4227b8562d61cb86198b0d3f2799948d` |
| `execution-framework/generated/execution_packets/REQ-042_FILESYSTEM_BOUNDS.json` | 2259 | `60e5c63a2a73958ca028be9fccfeacf29a1d487056dab32b99d295174d7e9f2b` |
| `execution-framework/generated/execution_packets/REQ-043_SECURITY_REDACTION.json` | 2266 | `1bfe9fe2cfcf6136d91432002cea128132b2b1330b0e5833e990c484df17198f` |
| `execution-framework/generated/execution_packets/REQ-044_INSTALL_BOOTSTRAP.json` | 2247 | `ab23eb4422028a2b144ffeecefdf78d2d38560ee90303b1a1e0cd10ea78d89a4` |
| `execution-framework/generated/execution_packets/REQ-045_RUN_REPLAY.json` | 2246 | `da77169223d2767602c138729386c6d7cfdca687a62fec16993bf4a9cb8aae04` |
| `execution-framework/generated/execution_packets/REQ-200_FLEXNETOS_TARGET_DESCRIPTOR.json` | 2407 | `27770552a124fb44fb17ddf4b4ddc1505be422fa9689368e6ab0ae5ad382e5a7` |
| `execution-framework/generated/execution_packets/REQ-201_FLEXNETOS_LIFEOS_COMPARISON.json` | 2531 | `39c72cdafd177a5c0f920d88c6b3e22358698694323af0f1ae3de1e0204ed3ef` |
| `execution-framework/generated/execution_packets/REQ-202_FLEXNETOS_ADAPTER_RECIPE.json` | 2390 | `f95cf386d5ec617d3fa7f897fb195f1421e1477457869486e08ed49d493d28b6` |
| `execution-framework/generated/execution_packets/VER-300_UNIT_VALIDATION.json` | 2390 | `caed4c7063f36a5e1b8e730edb022f2201c3cd8fa254f042202463b82491d38f` |
| `execution-framework/generated/execution_packets/VER-301_SQL_SCHEMA_TEST.json` | 2168 | `4166f35c6961c268626ccc818a1858fb5cc37d4ac9f134ced58ae4920cbe4b9c` |
| `execution-framework/generated/execution_packets/VER-302_PACKET_SCHEMA_VALIDATION.json` | 2376 | `be1c0332b71aec9c6e0d455fd7fabb49d7eed4666af59564dff14ce5cd0cc013` |
| `execution-framework/generated/execution_packets/VER-303_GOAL_LOOP_COMPUTE.json` | 2367 | `af597c2d1be3a3a3dccb75e42159a3f5b96622e5fb307f1413c8c647720a0128` |
| `execution-framework/generated/execution_packets/VER-304_FINAL_COMPLETENESS.json` | 2293 | `77c150b929c7c422bfb49b8d3c05000630229a3b282d6b0d8d388a5f0a6cb4fd` |
| `execution-framework/generated/final_verification_report.json` | 22032 | `9968dff1fa7dcdb584c81ac30a239a766c3b128ceb222cf12225dfeab0f13cc1` |
| `execution-framework/generated/gap_report.json` | 5774 | `3c5bf7490846f4eae56b1d1c1c97dbf44ebc00368393b9e176687f07ee9d1677` |
| `execution-framework/generated/package_manifest_addendum.json` | 364 | `9e1dd1fc158e672ff70759bbe843ca6b6f70e2b36a522f748209a916c66ab0ad` |
| `execution-framework/generated/package_scan.json` | 25864 | `0d851e9f6a67044bfd6b6df77231c518292a385304e1830d579e6520634b9487` |
| `execution-framework/generated/proof_ledger.merged.json` | 43130 | `7b39bf2b404268a57c7e7e17b818440a630cf02d03755896022dc013a9f0140c` |
| `execution-framework/generated/status_from_proofs.json` | 17452 | `49c079c20e5f9b556b5a45abd0a326636f1f525da4b03b775d15c89eeac04ab2` |
| `execution-framework/generated/status_report.json` | 18066 | `3862d5fe659ff46a3a7fe327ac58e2aaded001e45c50f37aa8ffa6d241b63d7e` |
| `execution-framework/generated/task_graph.csv` | 96251 | `409bc353bdc5dfe986ba444e81d338138c97213d11f8776275e0c570e2ababc8` |
| `execution-framework/generated/task_graph.index.json` | 6494 | `b1bd28d2b5596fe9555af86954f7c056999d42ff6c24259e3fd5237a7def7146` |
| `execution-framework/generated/task_graph.normalized.json` | 171125 | `69a790b3054c3e0865afcf9bd9c109b471d12a0f51255deb9bcf46fe3298c968` |
| `execution-framework/generated/task_graph.validation_report.json` | 148 | `2756e88e9e302ee9ee2b4519ce2799ce27eaf84a4d830d939aa608107b3e024c` |
| `execution-framework/generated/upgrade_report.json` | 12248 | `499f8bde020af5e895efc85da9c3170a0d1c64e4f1f5c131adea1cfb2a15d41c` |
| `execution-framework/proof_records/EF-001_SCAN_PACKAGE.proof.json` | 1152 | `ca3f56df9f2bc1a2642e756383946ef9f4064d32eea125ff461f156bed147e0c` |
| `execution-framework/proof_records/EF-002_GAP_REPORT.proof.json` | 1146 | `543834b6d54f3f45c5c4887e6fd9c6d90ac0f86fd8c0c3446d45f3791e55fae9` |
| `execution-framework/proof_records/EF-003_APPLY_FRAMEWORK.proof.json` | 1917 | `19370ede9f7220349b8b2baa534a19b72b6790b36da71ef8b36a5bd0d3ea1847` |
| `execution-framework/proof_records/EF-004_GENERATE_TASK_GRAPH.proof.json` | 1434 | `769ac572030fe834a7d092a4ea2b341df99b2a63c949ba829ff027d00423e428` |
| `execution-framework/proof_records/EF-005_VALIDATE_TASK_GRAPH.proof.json` | 1148 | `f7bdf419b6a42d1e260eb57c376a287f4bb2eaa32ab7c568af1d8fef1f7ffd2d` |
| `execution-framework/proof_records/EF-006_TASK_GRAPH_TO_PACKETS.proof.json` | 5948 | `dbf9394edfb8feb5481f550bd85275205f57f825db127ff0351ab82650d8c216` |
| `execution-framework/proof_records/EF-007_GOAL_LOOP.proof.json` | 1215 | `664991d4df2bdcf4622cd86f88f9d264973b84190c561a15dce9ff9cf16725b1` |
| `execution-framework/proof_records/EF-008_VERIFY_HISTORY_COMPLETENESS.proof.json` | 24751 | `d908d6d949e1bcd27a00b457be6921183bf32c39600826be4f821e1d7ee54f1c` |
| `execution-framework/proof_records/proof_ledger.jsonl` | 30579 | `e2e883176d1fed2280cd37a5650f92108afe972582f00bccaa2e546a0bb2a558` |
| `execution-framework/schemas/agent_lane.schema.json` | 997 | `e54c8c0ffade0a37babd9660053069145390957d566718e293cc8ede2b2b9d9f` |
| `execution-framework/schemas/execution_packet.schema.json` | 5999 | `a4305a40557ebea88f5699b6000000750563df9228916c21116b498e2e74a7e0` |
| `execution-framework/schemas/gap_report.schema.json` | 380 | `8889b598049ddf02f8f9fcd38a5504d3d20949d6975de1b7bc45fa1d16c6e647` |
| `execution-framework/schemas/proof_record.schema.json` | 1466 | `76c64fdf3f5943c8041610ee303b0e4cdcc1ba4321d9ade8e0f907a88a15f862` |
| `execution-framework/schemas/task_graph.schema.json` | 3002 | `2f3c8c1dec72a3317f0cfb8e419f0e5711bd61444cb65cdb89c8d3a9d6c111ea` |
| `execution-framework/schemas/upgrade_report.schema.json` | 392 | `fc5d58a6c441abf2e2fbfd86cb37859350d300924cfe6daeb07a512485e696d9` |
| `execution-framework/scripts/__pycache__/_common.cpython-313.pyc` | 8611 | `b0fedad35058452aea81e7effc9e0f37ded60db854142faca843e4db67a4fa4c` |
| `execution-framework/scripts/_common.py` | 4884 | `d2db3d2bccd23995cbbf1fe477c4b66847e9986b1f32b28da0dbe6238c757375` |
| `execution-framework/scripts/goal_loop.py` | 3984 | `7e99833ea0a385c736886c6807d4be94a93256f997faa82d788c9b348f39c1ea` |
| `execution-framework/scripts/goal_to_task_graph.py` | 904 | `a4696dd136f795b5ad3790fa2f786d13f4dbebdcae0466e68dd076d552413ce6` |
| `execution-framework/scripts/merge_proofs.py` | 446 | `c6bd47f3e1595afcfeacd10e0a640dacd9276e64be15dcad06c3d0de2fc751f2` |
| `execution-framework/scripts/scan_package.py` | 2052 | `b0404d9ba3a1fbc55204e926c442b9185b7da109c47da19b2b9e4b16aa926e8e` |
| `execution-framework/scripts/status_from_proofs.py` | 681 | `c11a5f03b2b3ce9b7838806fefef2846ed2347daa7f060268193f355b15f1330` |
| `execution-framework/scripts/task_graph_to_packets.py` | 2545 | `694e05731417dfa941ac2fcdf33bb64c46d718104a259626585d3288266be55f` |
| `execution-framework/scripts/validate_task_graph.py` | 4990 | `f09715ed1aaf59573ca7457052b2d75964511e220ab7e5c251870cb7e38e7223` |
| `execution-framework/scripts/verify_history_and_completeness.py` | 6788 | `a14d7b009018e16fcf90b5f27743e31e8ebb617e84525833eb4d29c724fd2766` |
| `execution-framework/state/goal_loop_state.json` | 18066 | `3862d5fe659ff46a3a7fe327ac58e2aaded001e45c50f37aa8ffa6d241b63d7e` |
| `execution-framework/templates/AGENT_LANE_TEMPLATE.json` | 624 | `fbac2f99486fe699114ab11b61da98e21c9be8a1e1ec4c3759f0f33b51d9164a` |
| `execution-framework/templates/TASK_GRAPH_TEMPLATE.csv` | 491 | `516dbead6806c0acd621622c98dbd2ef6c79e134caeba0aa13a1e1514ffeaf03` |
| `execution-framework/templates/TASK_GRAPH_TEMPLATE.md` | 738 | `f4afcbacebb1fb7f07737779fbbf351a0b21b5397848f85a82444921c9644e74` |
| `execution-templates/AGENT_LANE_TEMPLATE.json` | 624 | `fbac2f99486fe699114ab11b61da98e21c9be8a1e1ec4c3759f0f33b51d9164a` |
| `execution-templates/EXECUTION_TEMPLATE_INDEX.json` | 284 | `29ffeb26278c6e48a7954b7acb8e9570ece2a76e8aae89a77bb3efe4dc3e00c7` |
| `execution-templates/README.md` | 294 | `2cdd7c8549ffe0a49054909ba73f893dbc9c2dfa3a7bd28e42866582b5a4cab9` |
| `execution-templates/TASK_GRAPH_TEMPLATE.csv` | 491 | `516dbead6806c0acd621622c98dbd2ef6c79e134caeba0aa13a1e1514ffeaf03` |
| `execution-templates/two_repo_parallel_goal.yaml` | 889 | `286b45dd7eee83deb37a6d3a164d87eb3e5872beee46584be4de4fc2e2803234` |
| `history/README.md` | 296 | `4b82ddbae4ca50179c92eebeddef11e9f4b041aafea3ea8717224539e093550d` |
| `history/pre_execution_framework_manifest.json` | 22214 | `7f53ec278361330dbc4f5d5946398d4e36d16b3c6ba66af409656f5c3aaafcca` |
| `source/execution-framework-task-prompt.md` | 13606 | `43e6b928227d182c483b09d643c942ad263e706e19e065d2050687b5ab186f5a` |



## UPG-010 — Drive-live no-gap synchronization

Closed live bookkeeping gaps with maintenance packets/proofs, restored proof_templates, updated generated records, and added final Codex handoff prompt.
