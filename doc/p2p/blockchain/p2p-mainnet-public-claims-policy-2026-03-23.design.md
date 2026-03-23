# oasis7 mainnet/public claims policy 复评（设计文档）

- 对应需求文档: `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.project.md`

审计轮次: 1
## 设计目标
- 把 `MAINNET-1~3` 的规格 gate 结果收成不可漂移的 public claims policy。
- 明确当前仍是 preview，不允许任何 mainnet-grade/mint-ready 误升级。

## 当前复评结论
| 面向 | 当前状态 | public claims 结论 |
| --- | --- | --- |
| `MAINNET-1` signer custody | spec gate 完成，工程替换未完成 | 不得宣称 production signer custody 完成 |
| `MAINNET-2` governance signer | spec gate 完成，长期真值迁移未完成 | 不得宣称 governance signer externalization 完成 |
| `MAINNET-3` genesis gate | spec gate 完成，真实 binding/ceremony/QA pass 未完成 | 不得宣称 mint ready |
| 总体 | blockers 仍存在 | `not_mainnet_grade` |

## Claim Allowlist
- `limited playable technical preview`
- `crypto-hardened preview`
- `signed transaction model is implemented for exposed token actions`
- `mainnet-grade readiness gates are specified, but execution remains incomplete`

## Claim Denylist
- `mainnet-grade`
- `mainstream public-chain-grade security`
- `production mint ready`
- `genesis execution finalized`
- `production signer custody complete`

## Future Upgrade Conditions
1. production signer custody 实装并通过 QA。
2. governance signer long-term truth 外部化实装并通过 QA。
3. genesis slot/bucket 真值、ceremony 和最终 QA `pass` 全部完成。
4. 重新执行 producer/liveops 复评，才允许考虑升级口径。
