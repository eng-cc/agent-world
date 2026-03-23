# oasis7 治理 signer 外部化与轮换门禁（设计文档）

- 对应需求文档: `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.project.md`

审计轮次: 1
## 设计目标
- 把 current governance signer truth 从 local convenience 和 production governance target 两层拆开。
- 冻结 finality signer 与 controller signer 的长期真值、更新 authority 和失效恢复门禁。

## 当前治理 signer 真值
| Governance scope | 当前来源 | 当前问题 | 生产结论 |
| --- | --- | --- | --- |
| `finality signer` | `world/governance` deterministic local seed | 固定 seed label 派生，不具备正式治理更新能力 | preview-only |
| `controller signer` | `NodeConfig.main_token_controller_binding.controller_signer_policies` | 本地配置承担长期真值，缺 source-of-truth 分层 | partial/block |

## 目标态
| Governance scope | 目标真值 | 必需能力 | 禁止项 |
| --- | --- | --- | --- |
| `finality signer` | externalized governance signer registry | rotation、revocation、failover、operator ownership | deterministic local seed 进入 production |
| `controller signer` | externalized controller signer registry | threshold policy updates、rotation、revocation、audit | 仅靠单机 `NodeConfig` 维护生产真值 |

## Gate 切片
1. `GOVSIGN-1 inventory`: 固定 finality/controller signer 当前来源、环境等级和 blocker。
2. `GOVSIGN-2 truth boundary`: 冻结长期真值、update authority 与禁止项。
3. `GOVSIGN-3 ops policy`: 冻结 failover、rotation、revocation 与 operator ownership。
4. `GOVSIGN-4 release dependency`: 将 governance signer gate 接入 readiness/public-claims/ceremony 前置条件。

## 通过条件
- `MAINNET-2` 通过前必须满足：
  - finality/controller signer 都有明确长期真值。
  - local seed/config path 被明确限制在非 production。
  - failover/rotation/revocation/operator ownership 都有 gate。
  - readiness project 与模块主追踪同步更新。

## 对外口径
- 当前允许：
  - `crypto-hardened preview`
  - `signed governance controller proof exists in preview path`
- 当前禁止：
  - `production governance signer externalization is complete`
  - `mainnet-grade`
