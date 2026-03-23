# oasis7 创世 freeze / ceremony / QA gate（设计文档）

- 对应需求文档: `doc/p2p/blockchain/p2p-genesis-freeze-ceremony-qa-gate-2026-03-23.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-genesis-freeze-ceremony-qa-gate-2026-03-23.project.md`

审计轮次: 1
## 设计目标
- 把已有逻辑 freeze sheet 提升成真正的 mint 前门禁。
- 把 slot/bucket 真值、ceremony checklist 和 QA evidence bundle 串成单一事实源。

## 当前创世真值
| 面向 | 当前真值 | 当前问题 | 结论 |
| --- | --- | --- | --- |
| Freeze status | `logic_frozen_address_binding_pending` | 只冻结逻辑，没有冻结真实地址/控制者 | not_mint_ready |
| Slot registry | 多数 `current_value = TBD_BEFORE_MINT` 且 `status = pending_binding` | recipient/controller 真值未绑定 | block |
| Bucket execution sheet | 多数 `freeze_status = ready_pending_address_binding` | bucket 仍未形成正式执行真值 | block |
| QA gate | 仍要求后续跑 audit template，尚无最终 `pass` | 证据和 verdict 未形成 | block |

## 目标态
| 面向 | 目标状态 | 必需能力 |
| --- | --- | --- |
| Slot registry | `bound -> frozen` | recipient/controller 全量真实绑定 |
| Bucket execution sheet | `bound -> frozen` | signer policy/claim cadence/runtime target 全冻结 |
| Ceremony checklist | `executed -> evidenced` | operator checklist、binding proof、审计摘要 |
| QA verdict | `pass` | evidence bundle 完整、无阻断项 |

## Gate 切片
1. `GENESIS-1 freeze truth`: 固定 slot registry 与 bucket execution sheet 的放行条件。
2. `GENESIS-2 ceremony checklist`: 固定创世 ceremony 前中后 checklist 和 evidence 要求。
3. `GENESIS-3 QA bundle`: 固定 QA evidence bundle、verdict 模板与阻断结论。
4. `GENESIS-4 claim gate`: 把 mint-ready/public claims 绑定到 QA pass。

## 对外口径
- 当前允许：
  - `genesis logic is frozen`
  - `mint readiness is still blocked on binding/ceremony/QA`
- 当前禁止：
  - `production mint ready`
  - `genesis execution is finalized`
