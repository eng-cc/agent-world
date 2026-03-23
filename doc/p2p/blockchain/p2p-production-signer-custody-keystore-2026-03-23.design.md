# oasis7 生产级 signer custody / keystore 基线（设计文档）

- 对应需求文档: `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.project.md`

审计轮次: 1
## 设计目标
- 把当前 signer 来源从“能签”与“能进生产”两个层面拆开。
- 冻结 `node/viewer/governance` 三类 signer 的生产目标边界，作为 `MAINNET-1` 的完成标准。

## 当前 signer 真值
| Signer scope | 当前来源 | 当前问题 | 生产结论 |
| --- | --- | --- | --- |
| `node runtime signer` | `node_keypair_config` 自动生成并写回 `config.toml` | 私钥长期明文落地本地配置 | preview-only |
| `viewer/player signer` | `oasis7_web_launcher` 从 env 或 `config.toml` 读取私钥并注入 HTML | 页面级可见私钥 bootstrap，不适合生产 | preview-only |
| `governance/controller signer` | deterministic local seed 与本地 controller signer policy | 真值仍停留在 local/config，未托管、未轮换 | partial/block |

## 目标态
| Signer scope | 目标后端 | 必需能力 | 禁止项 |
| --- | --- | --- | --- |
| `node runtime signer` | offline storage + manual multisig | key isolation、rotation、revocation、audit trail、人工审批链 | 明文 `config.toml` 私钥 |
| `viewer/player signer` | delegated signing boundary；生产不保留页面私钥 | 页面不持有私钥、失败可观测、环境可控 | HTML 注入私钥、长期 env 私钥 |
| `governance/controller signer` | offline storage + manual multisig governance signers | rotation、revocation、operator ownership、人工审批链 | local seed/config 继续承担生产真值 |

## 选定方案备注
- producer 已选定 `offline storage + manual multisig` 作为当前生产 custody 路径。
- operator-local key staging/export root（例如 `~/Documents/keys`）仅作为 operator 本机的非仓库目录：
  - 不允许加入 git
  - 不允许被 runtime / web launcher 自动读取为 production key source
  - 不足以单独证明“真正离线”；仍需要后续人工 custody/runbook 约束

## Gate 切片
1. `CUSTODY-1 inventory`: 固定所有 signer scope 的当前来源、环境等级和 blocker。
2. `CUSTODY-2 target boundary`: 冻结每类 signer 的目标后端、调用边界与禁止项。
3. `CUSTODY-3 ops policy`: 冻结 rotation、revocation、audit trail 与 operator ownership。
4. `CUSTODY-4 release policy`: 冻结 local/dev/preview/production 的允许来源，并接入 release/public-claims gate。

## 通过条件
- `MAINNET-1` 通过前必须满足：
  - 三类 signer 都有明确生产目标边界。
  - preview bootstrap path 被明确限制在非 production 环境。
  - rotation/revocation/audit trail 都有 owner 与 gate。
  - readiness project 与模块主追踪同步更新。

## 对外口径
- 当前允许：
  - `crypto-hardened preview`
  - `signed transaction model is implemented for exposed token actions`
- 当前禁止：
  - `production signer custody is complete`
  - `mainnet-grade`
