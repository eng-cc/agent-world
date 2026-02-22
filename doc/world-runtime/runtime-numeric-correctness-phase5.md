# Agent World Runtime：Sequencer 主循环与 Lease 递进数值语义硬化（15 点清单第五阶段）

## 目标
- 收口 `agent_world_consensus` 中仍位于主链路的饱和递进语义，避免长期运行时出现静默夹逼后继续执行。
- 将 sequencer 的 `next_slot/next_height` 与 lease 的 `term/expires_at` 递进统一改为显式溢出失败。
- 保证溢出失败路径不产生局部状态污染（不做部分写入），并补齐可回归测试。

## 范围

### In Scope（第五阶段）
- `sequencer_mainloop`：
  - `next_slot` 递进从 `saturating_add(1)` 改为受检递进；越界时返回 `WorldError::DistributedValidationFailed`。
  - `next_height` 在 `Committed/Rejected` 终态递进从 `saturating_add(1)` 改为受检递进；越界时显式失败。
  - 终态应用改为“先计算后提交”，确保失败时本地状态不被部分更新。
- `lease`：
  - `next_term` 递进从 `saturating_add(1)` 改为受检递进。
  - `expires_at_ms = now_ms + ttl_ms` 从 `saturating_add` 改为受检加法（acquire/renew 两条路径）。
  - 越界时拒绝本次 acquire/renew，并保持原有 lease 状态。
- 测试：
  - 新增 sequencer/lease 边界溢出拒绝测试，覆盖“返回错误 + 状态未被部分更新”。

### Out of Scope（后续阶段）
- 全仓库统一数值新类型（如 `Slot/Height/Term` newtype）替换。
- 全量 `now_ms` 时间源统一与时钟漂移治理。
- 跨 crate 的大整数（BigInt/U256）协议演进。

## 接口/数据
- `SequencerMainloop`：
  - `tick(...) -> Result<SequencerTickReport, WorldError>` 签名不变。
  - 递进失败统一返回 `WorldError::DistributedValidationFailed`，包含字段上下文（如 `next_slot overflow`、`next_height overflow`）。
- `LeaseManager`：
  - 公开接口签名保持不变（`try_acquire` / `renew` 返回 `LeaseDecision`）。
  - 递进/到期时间越界转为 `LeaseDecision { granted: false, reason: Some(...) }`，并保证不污染现有 lease 状态。

## 里程碑
- M0：Phase5 文档建档并冻结范围。
- M1：sequencer 高度/slot 递进显式溢出语义落地。
- M2：lease term/ttl 递进显式溢出语义落地。
- M3：回归测试通过并完成文档/devlog 收口。

## 风险
- 行为从“饱和后继续执行”切换为“显式失败”，既有测试预期可能需要同步更新。
- sequencer 在极端边界可能进入“上限停机”语义，需要调用方将错误视为不可恢复并触发运维处置。
- lease 拒绝路径可观测性不足会影响排障，需要保持 `reason` 信息完整。

## 当前状态
- 截至 2026-02-23：M0 已完成，M1/M2/M3 进行中。
