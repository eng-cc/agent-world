# Agent World Runtime：Replication Writer Epoch/Sequence 数值语义硬化（15 点清单第四阶段）

## 目标
- 收口 `agent_world_node::replication` 中长期运行敏感的 writer `epoch/sequence` 递进语义。
- 将 writer 位置推进中的 `saturating_add` 改为显式溢出错误，避免静默饱和导致复制位置停滞或漂移。
- 在不改变复制协议格式的前提下，保证本地写入位置计算在越界时可观测、可测试、可拒绝。

## 范围

### In Scope（第四阶段）
- `ReplicationRuntime::next_local_record_position`：
  - 同 writer 递进 sequence 改为受检后继值。
  - writer 切换时 epoch 递进改为受检后继值。
  - 无 guard writer 时基于本地状态递进 sequence 改为受检后继值。
- `build_local_commit_message` 透传位置计算错误。
- 补齐 replication 模块溢出边界测试（至少覆盖同 writer sequence、writer 切换 epoch、无 guard sequence 三类）。
- 清理可直接确定安全的减法递进语义（去除非必要饱和减法）。

### Out of Scope（后续阶段）
- replication 协议版本升级与跨版本迁移。
- 全仓库统一数值新类型（`Epoch/Sequence` newtype）替换。
- 分布式复制流程形式化验证。

## 接口/数据
- `ReplicationRuntime::next_local_record_position`：
  - 从返回 `(u64, u64)` 改为 `Result<(u64, u64), NodeError>`。
  - 溢出时返回 `NodeError::Replication`，包含字段与上下文信息。
- 位置后继值：
  - 统一通过受检递进辅助函数计算，禁止静默饱和。

## 里程碑
- M0：Phase4 文档建档并冻结范围。
- M1：replication writer 位置递进显式溢出语义落地。
- M2：定向回归与文档收口。

## 风险
- 历史路径可能隐式依赖饱和语义，改造后边界会从“继续执行”切换到“显式失败”。
- 若仅覆盖单路径测试，仍可能漏掉 writer 切换与无 guard 的组合边界。

## 当前状态
- 截至 2026-02-23：M0 已完成，M1 进行中。
