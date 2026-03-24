# oasis7 shared network / release train 最小执行形态（设计文档）

- 对应需求文档: `doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.project.md`

审计轮次: 1
## 设计目标
- 把 benchmark 中 `L5 shared network/release train` 的缺口落成正式执行模型，而不是继续停留在口头 backlog。
- 明确 oasis7 下一阶段的最小 shared track、promotion 规则、rollback 规则与 claims gate。

## 当前结论
| 维度 | 当前状态 | 结论 |
| --- | --- | --- |
| local required/full + S6 + S9/S10 | 已具备 | `present` |
| governance drill clone/live evidence | 已具备基础 | `present_with_limited_coverage` |
| shared_devnet/staging/canary | 未正式建立 | `specified_not_executed` |
| public claims | 仍受 preview policy 限制 | `limited playable technical preview` + `crypto-hardened preview` |

## 三层 shared track
| Track | 目标 | 最小入口 | 最小通过标准 | 不算完成的情况 |
| --- | --- | --- | --- | --- |
| `shared_devnet` | 首次把统一 candidate 放到多人共享环境中运行 | 本地 gate 通过、candidate bundle 完整、共享访问路径明确 | 能被共享访问、版本固定、QA 有 `pass/block` 结论、可回滚到前一 bundle | 仍是单机私有 world、只有运行命令没有结论 |
| `staging` | 做升级窗口、恢复、回滚和彩排 | `shared_devnet=pass`、升级窗口与 owner 值班明确 | promotion/rollback 各至少一轮，证据完整，liveops 认可 | 只是复用 shared_devnet、没有独立升级/恢复演练 |
| `canary` | 小范围真实发布轨道，验证 freeze/incident 响应 | `staging=pass`、duration/freeze 条件/incident owner 明确 | 有固定观察窗、可执行 freeze/rollback、incident 结论闭环 | 没有观察窗、没有 incident 结论、没有 fallback bundle |

## Candidate Bundle
| 字段 | 说明 |
| --- | --- |
| `candidate_id` | 本次 release candidate 唯一标识 |
| `git_commit` | 对应仓库提交 |
| `runtime_build` | 运行时/构建产物标识 |
| `world_snapshot_ref` | world 真值引用 |
| `governance_manifest_ref` | governance 真值引用 |
| `evidence_refs` | 本地 gate、drill、QA 文档引用 |

## Promotion 规则
1. 任何 candidate 必须先完成本地 gate，再进入 `shared_devnet`。
2. 只有上一轨道结论为 `pass`，才允许 promotion 到下一轨道。
3. 一旦发现 commit/world/governance 真值漂移，立即 `freeze` 并退回重新编号。
4. `rollback` 目标必须是最近一次通过的 candidate bundle。

## Partial / Block 语义
| 状态 | 含义 |
| --- | --- |
| `pass` | 轨道目标与证据完整，允许推进 |
| `partial` | 有环境或运行，但缺 shared 访问、版本固定、QA 结论或 rollback 条件 |
| `block` | 未满足 promotion 必需条件，不允许推进 |
| `frozen` | 事故或漂移导致临时冻结推进 |
| `restored` | 已回滚到上一通过 bundle，并留下恢复证据 |

## 对外口径控制
- 当前允许：
  - `limited playable technical preview`
  - `crypto-hardened preview`
  - `shared network / release train is specified but not yet executed`
- 当前禁止：
  - `production release train is established`
  - `shared network validated`
  - `mainnet-grade testing maturity`
