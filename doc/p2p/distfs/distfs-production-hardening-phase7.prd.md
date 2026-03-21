# oasis7 Runtime：DistFS 生产化增强（Phase 7）设计文档

- 对应设计文档: `doc/p2p/distfs/distfs-production-hardening-phase7.design.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase7.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口为 `distfs-production-hardening-phase1.prd.md`，本文仅维护阶段增量。

## 1. Executive Summary
- Problem Statement: 在 DistFS 自适应挑战调度中引入按失败原因分级退避（reason-aware backoff），提高失败处理精度。
- Proposed Solution: 将自适应调度参数治理化到 `oasis7_viewer_live` CLI，支持运行时调优而无需改代码。
- Success Criteria:
  - SC-1: 在不突破单文件行数约束的前提下继续模块化 `oasis7_viewer_live`。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want oasis7 Runtime：DistFS 生产化增强（Phase 7）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH7-1：文档与任务拆解**
  - AC-2: 输出 Phase 7 设计文档与项目管理文档。
  - AC-3: **DPH7-2：Reason-aware 退避策略（DistFS）**
  - AC-4: 扩展 `StorageChallengeAdaptivePolicy`：
  - AC-5: `backoff_multiplier_hash_mismatch`;
  - AC-6: `backoff_multiplier_missing_sample`;
- Non-Goals:
  - 链上治理参数自动拉取。
  - 多节点集中式 challenge orchestrator。
  - PoRep/PoSt 升级。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase7.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase7.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 策略扩展（草案）
```rust
StorageChallengeAdaptivePolicy {
  max_checks_per_round: u32,
  failure_backoff_base_ms: i64,
  failure_backoff_max_ms: i64,
  backoff_multiplier_hash_mismatch: u32,
  backoff_multiplier_missing_sample: u32,
  backoff_multiplier_timeout: u32,
  backoff_multiplier_read_io_error: u32,
  backoff_multiplier_signature_invalid: u32,
  backoff_multiplier_unknown: u32,
}
```

### CLI 参数（草案）
```text
--reward-distfs-adaptive-max-checks-per-round <u32, >0>
--reward-distfs-adaptive-backoff-base-ms <i64, >=0>
--reward-distfs-adaptive-backoff-max-ms <i64, >=0, >=base>
```

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH7-M1**：文档与任务拆解完成。
  - **DPH7-M2**：reason-aware 退避策略完成。
  - **DPH7-M3**：CLI 参数治理化与接线完成。
  - **DPH7-M4**：回归与文档收口完成。
- Technical Risks:
  - 参数组合过多会增加运维配置复杂度；通过默认策略与严格校验降低误配风险。
  - reason 分类若不稳定可能造成退避抖动；本期沿用稳定失败原因枚举。
  - CLI 扩展可能推高主文件复杂度；通过模块化拆分控制技术债。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-073-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-073-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
