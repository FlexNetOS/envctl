# Envctl Migration Live Visuals

Generated: 2026-07-04T23:09:09+00:00

## Overview

tasks | complete | ready | blocked | failed | proofs | missing proofs
------+----------+-------+---------+--------+--------+---------------
80    | 16       | 8     | 56      | 0      | 12     | 68            

## Parallel Lanes

lane                | tasks | done | ready | blocked | pending | active                                                                                                                                                         
--------------------+-------+------+-------+---------+---------+----------------------------------------------------------------------------------------------------------------------------------------------------------------
lane_a_planning     | 3     | 3    | 0     | 0       | 0       | -                                                                                                                                                              
lane_b_repo_a       | 10    | 1    | 5     | 0       | 9       | REQ-021_ENVCTL_TARGET_REGISTRY, REQ-022_ENVCTL_RUN_LEDGER, REQ-023_ENVCTL_OPERATION_STATE, REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-025_ENVCTL_VALIDATION_EVIDENCE
lane_c_repo_b       | 5     | 2    | 2     | 0       | 3       | REQ-031_PLUGIN_COMMAND_SURFACE, REQ-034_PLUGIN_STATUS_STREAMS                                                                                                  
lane_d_filesystem   | 45    | 1    | 1     | 0       | 44      | REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                                
lane_e_verification | 14    | 8    | 0     | 0       | 6       | -                                                                                                                                                              
lane_f_release      | 2     | 0    | 0     | 0       | 2       | -                                                                                                                                                              
lane_g_history_scan | 1     | 1    | 0     | 0       | 0       | -                                                                                                                                                              

## Parallel Groups

group                  | tasks | max | ready | capacity | done | blocked
-----------------------+-------+-----+-------+----------+------+--------
artifact_generation_a  | 12    | 6   | 0     | 0        | 0    | 0      
artifact_generation_b  | 12    | 6   | 0     | 0        | 0    | 0      
artifact_generation_c  | 13    | 6   | 0     | 0        | 0    | 0      
contract               | 1     | 1   | 0     | 0        | 1    | 0      
drive_live_maintenance | 4     | 2   | 0     | 0        | 4    | 0      
envctl_db_parallel     | 6     | 4   | 5     | 4        | 0    | 0      
envctl_integration     | 2     | 2   | 0     | 0        | 0    | 0      
filesystem_parallel    | 4     | 3   | 0     | 0        | 0    | 0      
flexnetos              | 3     | 1   | 0     | 0        | 0    | 0      
framework_bootstrap    | 4     | 1   | 0     | 0        | 4    | 0      
framework_verification | 4     | 1   | 0     | 0        | 4    | 0      
integration            | 1     | 1   | 0     | 0        | 0    | 0      
nu_plugin_parallel     | 4     | 4   | 2     | 2        | 1    | 0      
release                | 2     | 1   | 0     | 0        | 0    | 0      
shared_protocol        | 3     | 2   | 1     | 1        | 2    | 0      
verification           | 5     | 1   | 0     | 0        | 0    | 0      

## Proof State

proof status | count
-------------+------
completed    | 16   
missing      | 64   

## Blockers

task                                | status  | lane                | reason                                      | unmet deps                                                                                                                                                                                   
------------------------------------+---------+---------------------+---------------------------------------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS | pending | lane_b_repo_a       | human approval required                     | -                                                                                                                                                                                            
REQ-027_ENVCTL_REPLAY_ENGINE        | pending | lane_b_repo_a       | unmet dependencies, human approval required | REQ-021_ENVCTL_TARGET_REGISTRY, REQ-022_ENVCTL_RUN_LEDGER, REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS                                                                                               
REQ-028_ENVCTL_AGENT_CONTROL_API    | pending | lane_b_repo_a       | unmet dependencies, human approval required | REQ-023_ENVCTL_OPERATION_STATE, REQ-025_ENVCTL_VALIDATION_EVIDENCE                                                                                                                           
REQ-033_PLUGIN_HUMAN_APPROVAL       | pending | lane_c_repo_b       | human approval required                     | -                                                                                                                                                                                            
REQ-041_TWO_REPO_INTEGRATION        | pending | lane_e_verification | unmet dependencies, human approval required | REQ-027_ENVCTL_REPLAY_ENGINE, REQ-028_ENVCTL_AGENT_CONTROL_API, REQ-031_PLUGIN_COMMAND_SURFACE, REQ-033_PLUGIN_HUMAN_APPROVAL, REQ-034_PLUGIN_STATUS_STREAMS, REQ-040_SHARED_PROTOCOL_SCHEMAS
REQ-042_FILESYSTEM_BOUNDS           | pending | lane_d_filesystem   | unmet dependencies                          | REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                                                              
REQ-043_SECURITY_REDACTION          | pending | lane_d_filesystem   | unmet dependencies                          | REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                                                              
REQ-044_INSTALL_BOOTSTRAP           | pending | lane_d_filesystem   | unmet dependencies                          | REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                                                              
REQ-045_RUN_REPLAY                  | pending | lane_d_filesystem   | unmet dependencies, human approval required | REQ-044_INSTALL_BOOTSTRAP, REQ-027_ENVCTL_REPLAY_ENGINE                                                                                                                                      
ART-100_SYSTEM_INVENTORY            | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-101_DIRECTORY_TREE              | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-102_REPOSITORY_MAP              | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-103_SERVICE_DEP_GRAPH           | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-104_TOOLCHAIN_TREE              | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-105_PACKAGE_LIB_GRAPH           | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-106_RUNTIME_DEP_MAP             | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-107_DATA_FLOW_GRAPH             | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-108_DB_SCHEMA_MAP               | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-109_DATA_LINEAGE                | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-110_API_CATALOG                 | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-111_EVENT_MAP                   | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-112_CODE_OWNERSHIP              | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-113_DEBUG_CODE_MAP              | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-114_ENV_CONFIG_MATRIX           | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-115_CONFIG_INVENTORY            | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-116_INFRA_TOPOLOGY              | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-117_IAM_MATRIX                  | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-118_OBSERVABILITY               | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-119_BUSINESS_PROCESS            | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            
ART-120_WAVE_PLAN                   | pending | lane_d_filesystem   | unmet dependencies                          | REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS                                                                                                                            

## Graph Edges

from                                | to                                  | type      
------------------------------------+-------------------------------------+-----------
EF-001_SCAN_PACKAGE                 | EF-002_GAP_REPORT                   | depends_on
EF-002_GAP_REPORT                   | EF-003_APPLY_FRAMEWORK              | depends_on
EF-003_APPLY_FRAMEWORK              | EF-004_GENERATE_TASK_GRAPH          | depends_on
EF-004_GENERATE_TASK_GRAPH          | EF-005_VALIDATE_TASK_GRAPH          | depends_on
EF-005_VALIDATE_TASK_GRAPH          | EF-006_TASK_GRAPH_TO_PACKETS        | depends_on
EF-006_TASK_GRAPH_TO_PACKETS        | EF-007_GOAL_LOOP                    | depends_on
EF-007_GOAL_LOOP                    | EF-008_VERIFY_HISTORY_COMPLETENESS  | depends_on
EF-008_VERIFY_HISTORY_COMPLETENESS  | REQ-010_CONTRACT_LOCK               | depends_on
REQ-010_CONTRACT_LOCK               | REQ-020_ENVCTL_DB_SCHEMA            | depends_on
REQ-020_ENVCTL_DB_SCHEMA            | REQ-021_ENVCTL_TARGET_REGISTRY      | depends_on
REQ-020_ENVCTL_DB_SCHEMA            | REQ-022_ENVCTL_RUN_LEDGER           | depends_on
REQ-020_ENVCTL_DB_SCHEMA            | REQ-023_ENVCTL_OPERATION_STATE      | depends_on
REQ-020_ENVCTL_DB_SCHEMA            | REQ-024_ENVCTL_ARTIFACT_REGISTRY    | depends_on
REQ-020_ENVCTL_DB_SCHEMA            | REQ-025_ENVCTL_VALIDATION_EVIDENCE  | depends_on
REQ-020_ENVCTL_DB_SCHEMA            | REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS | depends_on
REQ-021_ENVCTL_TARGET_REGISTRY      | REQ-027_ENVCTL_REPLAY_ENGINE        | depends_on
REQ-022_ENVCTL_RUN_LEDGER           | REQ-027_ENVCTL_REPLAY_ENGINE        | depends_on
REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS | REQ-027_ENVCTL_REPLAY_ENGINE        | depends_on
REQ-023_ENVCTL_OPERATION_STATE      | REQ-028_ENVCTL_AGENT_CONTROL_API    | depends_on
REQ-025_ENVCTL_VALIDATION_EVIDENCE  | REQ-028_ENVCTL_AGENT_CONTROL_API    | depends_on
REQ-010_CONTRACT_LOCK               | REQ-030_PLUGIN_PROTOCOL_MANIFEST    | depends_on
REQ-030_PLUGIN_PROTOCOL_MANIFEST    | REQ-031_PLUGIN_COMMAND_SURFACE      | depends_on
REQ-030_PLUGIN_PROTOCOL_MANIFEST    | REQ-032_PLUGIN_LIVE_VISUALS         | depends_on
REQ-030_PLUGIN_PROTOCOL_MANIFEST    | REQ-033_PLUGIN_HUMAN_APPROVAL       | depends_on
REQ-030_PLUGIN_PROTOCOL_MANIFEST    | REQ-034_PLUGIN_STATUS_STREAMS       | depends_on
REQ-010_CONTRACT_LOCK               | REQ-040_SHARED_PROTOCOL_SCHEMAS     | depends_on
REQ-027_ENVCTL_REPLAY_ENGINE        | REQ-041_TWO_REPO_INTEGRATION        | depends_on
REQ-028_ENVCTL_AGENT_CONTROL_API    | REQ-041_TWO_REPO_INTEGRATION        | depends_on
REQ-031_PLUGIN_COMMAND_SURFACE      | REQ-041_TWO_REPO_INTEGRATION        | depends_on
REQ-032_PLUGIN_LIVE_VISUALS         | REQ-041_TWO_REPO_INTEGRATION        | depends_on
REQ-033_PLUGIN_HUMAN_APPROVAL       | REQ-041_TWO_REPO_INTEGRATION        | depends_on
REQ-034_PLUGIN_STATUS_STREAMS       | REQ-041_TWO_REPO_INTEGRATION        | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | REQ-041_TWO_REPO_INTEGRATION        | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | REQ-042_FILESYSTEM_BOUNDS           | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | REQ-043_SECURITY_REDACTION          | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | REQ-044_INSTALL_BOOTSTRAP           | depends_on
REQ-044_INSTALL_BOOTSTRAP           | REQ-045_RUN_REPLAY                  | depends_on
REQ-027_ENVCTL_REPLAY_ENGINE        | REQ-045_RUN_REPLAY                  | depends_on
REQ-024_ENVCTL_ARTIFACT_REGISTRY    | ART-100_SYSTEM_INVENTORY            | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | ART-100_SYSTEM_INVENTORY            | depends_on
REQ-024_ENVCTL_ARTIFACT_REGISTRY    | ART-101_DIRECTORY_TREE              | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | ART-101_DIRECTORY_TREE              | depends_on
REQ-024_ENVCTL_ARTIFACT_REGISTRY    | ART-102_REPOSITORY_MAP              | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | ART-102_REPOSITORY_MAP              | depends_on
REQ-024_ENVCTL_ARTIFACT_REGISTRY    | ART-103_SERVICE_DEP_GRAPH           | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | ART-103_SERVICE_DEP_GRAPH           | depends_on
REQ-024_ENVCTL_ARTIFACT_REGISTRY    | ART-104_TOOLCHAIN_TREE              | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | ART-104_TOOLCHAIN_TREE              | depends_on
REQ-024_ENVCTL_ARTIFACT_REGISTRY    | ART-105_PACKAGE_LIB_GRAPH           | depends_on
REQ-040_SHARED_PROTOCOL_SCHEMAS     | ART-105_PACKAGE_LIB_GRAPH           | depends_on

