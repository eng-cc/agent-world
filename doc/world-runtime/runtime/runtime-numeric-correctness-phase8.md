# Agent World Runtime：Membership Dead-Letter Replay 重试计数与比率阈值数值语义硬化（15 点清单第八阶段）

## 目标
- 收口 `membership_recovery` 主链路中仍存在的 `saturating_*` 数值语义，避免长期运行下出现“静默夹逼后继续执行”。
- 对重试失败路径中的 `attempt + 1` 与 `now_ms + retry_backoff_ms` 采用显式受检语义，越界时返回可观测错误并保持状态不被部分写入。
- 对自适应策略中的“乘法阈值比较”和“比例换算”去饱和化，确保极端大整数输入下决策语义与数学定义一致。

## 范围

### In Scope（第八阶段）
- `crates/agent_world_consensus/src/membership_recovery/types.rs`
  - `MembershipRevocationPendingAlert::with_retry_failure` 从饱和递进改为受检递进：
    - `attempt` 递进受检；
    - `next_retry_at_ms = now_ms + retry_backoff_ms` 受检。
- `crates/agent_world_consensus/src/membership_recovery/mod.rs`
  - 接线重试失败路径，透传数值越界错误；
  - 保证越界失败时 pending/dead-letter 不发生部分更新。
- `crates/agent_world_consensus/src/membership_recovery/replay.rs`
  - 自适应策略中 `* 2` 阈值比较与 `* 1000` 比率换算改为无溢出实现（避免 `saturating_mul` 失真）。
- 测试：
  - 新增重试计数/回退时间越界拒绝测试；
  - 新增极端大整数策略比率边界测试，验证不会因饱和乘法误判。

### Out of Scope（后续阶段）
- membership 子系统全量计数器 newtype 化。
- 跨 crate 时间源统一与全链路时钟漂移治理。
- dead-letter 治理策略本身的产品语义重设计（仅做数值正确性收口）。

## 接口/数据
- `MembershipRevocationPendingAlert::with_retry_failure(...)`：
  - 由直接返回 `Self` 改为 `Result<Self, WorldError>`（或等价可失败接口），越界返回 `WorldError::DistributedValidationFailed`。
- `recommend_revocation_dead_letter_replay_policy` 相关内部计算：
  - 不改变公开 API；
  - 仅替换内部阈值与比率计算方式，确保无溢出判定。

## 里程碑
- M0：Phase8 建档并冻结范围。
- M1：重试计数/回退时间受检递进落地。
- M2：自适应策略比率与阈值去饱和化落地并补测试。
- M3：回归测试通过，文档与 devlog 收口。

## 风险
- 失败语义从“静默夹逼继续执行”切换为“显式失败”，会改变极端边界下原有行为预期。
- 若调用链仅局部接线可失败接口，可能导致语义分裂，需要一次性改齐。
- 策略阈值比较实现方式调整后，边界断言需同步更新，避免测试期望沿用旧语义。

## 当前状态
- 截至 2026-02-23：M0、M1、M2、M3 全部完成。
