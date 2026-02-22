# Agent World Runtime：共识数值语义与原子状态转移硬化（15 点清单第一阶段）

## 目标
- 以“正确性优先”为原则落地数值安全与状态一致性治理，作为 15 点长期清单的第一阶段。
- 消除链上状态转移中的静默饱和与半提交风险，确保错误发生时显式失败且不污染状态。
- 建立后续 15 点全量改造的分期锚点，避免后续改动偏离统一语义。

## 范围

### In Scope（第一阶段）
- 数值语义收敛（先覆盖 runtime 账本主路径）：
  - `ResourceStock::add` 从饱和加法改为显式溢出错误。
  - `ResourceDelta` 聚合从“静默饱和”改为“可报告的溢出错误”。
  - `World::adjust_resource_balance`/`apply_resource_delta` 在溢出时返回错误而非截断。
- 状态转移原子性（先覆盖高风险事件）：
  - `DomainEvent::ResourceTransferred`
  - `DomainEvent::PowerRedeemed`
  - `DomainEvent::EconomicContractSettled`
  - 统一采用“先校验与预计算，再提交写入”的模式，避免先扣后加失败。
- 执行流接线：
  - `step_with_modules` 遇到资源 delta 溢出时，落 `ActionRejected`，不中断主循环，不提交部分状态。
- 测试：
  - 增加溢出拒绝测试与原子性回归测试。

### Out of Scope（后续阶段）
- 全仓库领域数值新类型（`Height/Slot/Epoch/Credit` 等）替换。
- 全链路 canonical 编码与签名域版本治理改造。
- 形式化验证（TLA+/Coq）与跨节点确定性验证平台化。
- 全仓库 `as`/裸算术 CI Gate 一次性收口。

## 接口/数据
- `ResourceStock::add`：
  - 输入仍为 `(kind, amount)`，但 `amount > 0` 时使用 `checked_add`。
  - 溢出返回 `StockError::Overflow`（新增）。
- `ResourceDelta::add_assign`：
  - 返回 `Result<(), ResourceDeltaOverflowError>`（从无返回值改为可失败）。
  - `merge_rule_decisions` 将该错误映射为 `RuleDecisionMergeError::CostOverflow`（新增）。
- `World::adjust_resource_balance`：
  - 返回 `Result<i64, WorldError>`（从 `i64` 改为可失败）。
- `World::apply_resource_delta`：
  - 返回 `Result<(), WorldError>`（从无返回值改为可失败）。
- 事件应用策略：
  - 先读取并校验来源余额、目标余额上界、配额/nonce 等。
  - 再执行写入，写入阶段不再触发可预见失败路径。

## 里程碑
- M0：设计/项目文档建档并冻结第一阶段边界。
- M1：数值语义改造（显式溢出错误）落地。
- M2：三条高风险事件原子化提交落地。
- M3：执行流拒绝路径与回归测试通过。
- M4：文档/devlog 收口并为第二阶段准备输入。

## 风险
- 改造后行为从“饱和成功”切换为“显式失败”，可能导致旧测试预期变化。
- 事件应用原子化会提升单事件校验复杂度，需要防止校验与提交条件不一致。
- 某些历史数据若接近边界，改造后会进入拒绝路径，需要日志可观测与错误码明确。

## 15 点清单映射（阶段视角）
- 本阶段直接覆盖：2、3、5、6、14（局部）。
- 下一阶段优先覆盖：1、4、8、11。
- 后续阶段覆盖：7、9、10、12、13、15。
