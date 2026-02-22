# Agent World Runtime：Membership 协调租约与时间源窄化数值语义硬化（15 点清单第六阶段）

## 目标
- 收口 `agent_world_consensus` 中 membership 协调租约主路径的饱和加法语义，避免长期运行后出现静默夹逼并继续调度。
- 收口 consensus/net/node 中 `Duration::as_millis() as i64` 的窄化转换风险，避免极端运行时长下的隐式截断。
- 保持“失败不部分写入”原则：数值计算失败时拒绝当前操作，不污染已有状态。

## 范围

### In Scope（第六阶段）
- membership 协调器 lease 到期时间计算：
  - `InMemoryMembershipRevocationScheduleCoordinator::acquire` 中 `now_ms + lease_ttl_ms` 从 `saturating_add` 改为受检加法。
  - `StoreBackedMembershipRevocationScheduleCoordinator::acquire` 中 `now_ms + lease_ttl_ms` 从 `saturating_add` 改为受检加法。
  - 溢出时返回 `WorldError::DistributedValidationFailed`，并保证 lease 状态不写入/不覆盖。
- 时间源毫秒转换：
  - 将 phase6 范围内 `duration.as_millis() as i64` 改为受检转换（统一 helper 语义）。
  - 若毫秒值超出 `i64` 可表示范围，返回明确错误（可失败接口）或在既有不可失败接口中采用可观测的上限夹逼语义。
- 测试：
  - 新增 membership coordinator 溢出拒绝与状态不变测试（in-memory + store-backed）。
  - 新增时间源转换边界测试（至少覆盖超大 `Duration` 的行为约束）。

### Out of Scope（后续阶段）
- 跨 crate 全量时间/高度/slot/term newtype 化重构。
- 协议层引入 U256/BigInt 并改变线上状态编码格式。
- 非主链路统计计数器（监控类 `saturating_*`）的全量语义收口。

## 接口/数据
- membership 协调器：
  - 公开 trait 签名保持不变：`acquire(...) -> Result<bool, WorldError>`。
  - 仅改变边界语义：时间计算越界返回 `Err(WorldError::DistributedValidationFailed { ... })`。
- 时间源 helper：
  - 新增统一毫秒转换 helper（按模块可见性放置），用于替代 `as_millis() as i64`。
  - helper 语义为“受检转换 + 明确失败信息”或“受检转换 + 可观测上限夹逼”，依据调用点错误模型保持兼容。

## 里程碑
- M0：Phase6 建档并冻结范围。
- M1：membership 协调器 lease 递进受检语义改造完成。
- M2：`as_millis() as i64` 风险点完成受检转换收口。
- M3：回归测试通过并完成文档/devlog 收口。

## 风险
- membership acquire 从“静默夹逼成功”改为“显式失败”，可能改变极端边界下调度可用性表现。
- 时间源转换语义调整后，部分调用点的时间值可能由“隐式截断”变为“失败/上限值”，需要测试同步更新。
- 若 net/node 中存在不可失败路径，需谨慎选择夹逼语义并保证可观测性，避免二次静默风险。

## 当前状态
- 截至 2026-02-23：M0、M1 已完成；M2、M3 进行中。
