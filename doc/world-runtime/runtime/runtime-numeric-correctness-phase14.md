# Agent World Runtime：Membership Sync 与 Mempool 批处理计数算术语义硬化（15 点清单第十四阶段）

## 目标
- 收口 `membership.rs` 与 `mempool.rs` 中剩余的关键计数/容量饱和算术，统一到受检语义。
- 在长期运行与极端边界下，将“静默饱和继续执行”升级为“显式失败且不污染状态”。
- 对齐前序阶段的错误语义：数值越界统一返回 `WorldError::DistributedValidationFailed` 并携带上下文。

## 范围

### In Scope（第十四阶段）
- `crates/agent_world_consensus/src/membership.rs`
  - `sync_key_revocations` / `sync_key_revocations_with_policy` / `sync_membership_directory` 的关键计数累加从 `saturating_add` 改为受检累加。
  - 新增受检计数 helper，统一错误语义与错误消息。
- `crates/agent_world_consensus/src/mempool.rs`
  - `take_batch_with_rules` 的 `total_bytes + size_bytes` 从饱和累加改为受检累加。
  - 保持失败路径原子性：若发生溢出，不移除 mempool 内现有 action。
- 测试
  - 为新增 helper 增加 overflow 单元测试。
  - 复用并扩展现有 membership/mempool 测试，验证改造后语义与状态一致性。

### Out of Scope（后续阶段）
- `membership_recovery/replay*.rs` 中策略层面非关键饱和算术收口。
- `node_pos.rs` / `pos.rs` / `sequencer_mainloop.rs` 中基于 epoch 的非关键饱和减法语义重构。
- 全仓库统一数值 newtype 与跨 crate 数值 lint gate。

## 接口/数据
- 对外 API 入口保持不变（继续返回 `Result<..., WorldError>`）。
- 内部增加受检 helper（`checked_usize_add` / `checked_usize_increment`），替换关键 `saturating_add`。
- 失败模型统一：
  - `WorldError::DistributedValidationFailed`
  - 错误消息包含字段上下文（如 `lhs/rhs`、计数字段名）。

## 里程碑
- M0：Phase14 建档并冻结边界。
- M1：Membership Sync 关键计数受检改造完成。
- M2：Mempool payload 累加受检改造与边界测试完成。
- M3：回归测试通过并完成文档/devlog 收口。

## 风险
- 从饱和语义切换到显式失败后，边界行为将从“容忍继续执行”变为“拒绝”，需同步更新测试预期。
- `take_batch_with_rules` 若在边界条件返回错误，需要确认不发生 action 提前移除。
- 计数字段较分散，需避免部分路径遗漏导致统计不一致。

## 当前状态
- 截至 2026-02-23：M0、M1、M2、M3 已完成（Phase14 收口完成）。
