# Shared Network Track Gate 模板

审计轮次: 1

## 目的
- 为 `RTMIN-2` 提供统一的 shared-network QA gate 模板。
- 固定 `shared_devnet`、`staging`、`canary` 三层轨道的最小 lane、`pass/partial/block` 判定与证据回写格式。
- 让 `qa_engineer`、`runtime_engineer`、`liveops_community` 在同一 candidate bundle 上给出一致结论。

## 使用说明
- 先生成并校验同一份 `release_candidate_bundle`。
- 再按当前 track 填写对应 lanes TSV，并用 `./scripts/shared-network-track-gate.sh` 生成 `summary.json/md`。
- 若缺失 required lane，脚本会直接把 gate 结论写成 `block`。
- 同一份 gate 只对应一个 `track + candidate_id`；不得混用不同候选或不同 world/governance 真值。

## 结论状态
| 状态 | 含义 |
| --- | --- |
| `pass` | required lanes 全部具备，且所有 lane 结论均为 `pass` |
| `partial` | required lanes 全部具备，但至少一条 lane 仍为 `partial` |
| `block` | 缺 required lane，或任一 lane 明确 `block` |

## Track Required Lanes
| Track | Required lanes |
| --- | --- |
| `shared_devnet` | `candidate_bundle_integrity` / `shared_access` / `multi_entry_closure` / `governance_live_drill` / `short_window_longrun` / `rollback_target_ready` |
| `staging` | `candidate_bundle_integrity` / `shared_access` / `unified_candidate_gate` / `governance_live_drill` / `upgrade_rehearsal` / `rollback_rehearsal` / `incident_template` |
| `canary` | `candidate_bundle_integrity` / `promotion_record` / `canary_window` / `rollback_rehearsal` / `incident_review` / `exit_decision` |

## TSV 列格式
```tsv
lane_id<TAB>owner<TAB>status<TAB>evidence_path<TAB>note
```

## 模板文件
- `shared_devnet`: `doc/testing/templates/shared-network-track-gate-lanes.shared_devnet.template.tsv`
- `staging`: `doc/testing/templates/shared-network-track-gate-lanes.staging.template.tsv`
- `canary`: `doc/testing/templates/shared-network-track-gate-lanes.canary.template.tsv`

## 最小命令
```bash
./scripts/shared-network-track-gate.sh \
  --track shared_devnet \
  --candidate-bundle output/release-candidates/shared-devnet-01.json \
  --lanes-tsv doc/testing/templates/shared-network-track-gate-lanes.shared_devnet.template.tsv \
  --out-dir output/shared-network-gates
```

## 审查清单
- 是否所有 lane 都引用同一 candidate bundle。
- 是否所有 evidence path 都可达。
- 是否 track 对应 required lanes 已全部填写。
- 是否 `pass/partial/block` 与证据内容一致。
- 是否已回写 `doc/p2p/blockchain/*project.md`、`testing-manual.md` 与 `doc/devlog/YYYY-MM-DD.md`。
