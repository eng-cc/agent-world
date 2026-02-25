# Non-Viewer 设计一致性修复（2026-02-25）

## 目标
- 修复 non-viewer 两处“实现偏离设计语义”的问题，确保线上行为与文档约束一致：
  - 共识 rejected 分支不再静默丢弃回灌失败的 action。
  - dead-letter file store 在冷热分层后仍可通过统一接口回放完整记录。
- 保持既有内存边界、冷归档策略与兼容性，不引入 Viewer 相关变更。

## 范围

### In Scope
- `crates/agent_world_node`
  - `PosNodeEngine::apply_decision` 的 rejected 回灌错误处理。
  - `pending_consensus_action_capacity` 增加 pending proposal 回灌容量预留。
  - 补充回归测试（拒绝分支回灌溢出显式报错、容量预留语义）。
- `crates/agent_world_consensus`
  - `FileMembershipRevocationAlertDeadLetterStore` 的 `list`/`list_delivery_metrics` 改为冷热聚合读取。
  - `replace` 同步清理并重建 archive refs，避免 stale cold refs。
  - 补充回归测试（replace 后冷归档无陈旧数据）。

### Out of Scope
- 共识协议语义调整（PoS 规则、投票机制）。
- Viewer 代码、Web/UI 交互路径。
- dead-letter replay 策略参数调优。

## 接口 / 数据

### 1) Rejected action 回灌
- 行为变更：`PosNodeEngine` 在 rejected 分支回灌失败时返回显式 `NodeError::Consensus`，不再吞错。
- 容量语义：`pending_consensus_action_capacity` 计算时预留 `pending proposal committed_actions.len()` 的回灌空间。

### 2) Dead-letter 冷热回放
- `FileMembershipRevocationAlertDeadLetterStore::list`
  - 从“仅热文件读取”改为“cold refs + hot 文件聚合读取”。
- `FileMembershipRevocationAlertDeadLetterStore::list_delivery_metrics`
  - 同步改为冷热聚合读取。
- `FileMembershipRevocationAlertDeadLetterStore::replace`
  - 写入剩余记录前清理历史 archive refs，再按 retention 重建冷热分层。

## 里程碑
- M0：建档（设计 + 项管）完成。
- M1：Node rejected 回灌安全修复 + 回归测试通过。
- M2：Dead-letter 冷热回放与 replace 修复 + 回归测试通过。
- M3：文档/devlog 收口。

## 风险
- 冷热聚合读取会比仅热读取成本更高。
  - 缓解：保留 retention 与冷段分片，避免单文件无限增长。
- replace 重建 archive refs 后会留下不可达 CAS 旧 blob。
  - 缓解：不影响正确性；后续可通过离线 GC 收敛空间。

## 当前状态
- 状态：已完成（2026-02-25）
- 已完成：M0、M1、M2、M3
- 阻塞项：无
