# oasis7 Runtime：DistFS 生产化增强（Phase 8）设计文档

- 对应设计文档: `doc/p2p/distfs/distfs-production-hardening-phase8.design.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase8.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口为 `distfs-production-hardening-phase1.prd.md`，本文仅维护阶段增量。

## 1. Executive Summary
- Problem Statement: 将 reason-aware 退避策略中的失败原因倍率参数（multiplier）完整治理化到 `world_viewer_live` CLI。
- Proposed Solution: 让 reward runtime 运行时可按失败原因分层调优退避，不再依赖编译期默认值。
- Success Criteria:
  - SC-1: 保持主入口文件规模可控并补齐参数解析与序列化观测测试。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want oasis7 Runtime：DistFS 生产化增强（Phase 8）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH8-1：文档与任务拆解**
  - AC-2: 输出 Phase 8 设计文档与项目管理文档。
  - AC-3: **DPH8-2：Multiplier 参数治理化（CLI + Runtime）**
  - AC-4: 为如下字段新增 CLI 参数并接入 `DistfsProbeRuntimeConfig.adaptive_policy`：
  - AC-5: `backoff_multiplier_hash_mismatch`
  - AC-6: `backoff_multiplier_missing_sample`
- Non-Goals:
  - 动态链上配置下发 multiplier。
  - 多策略模板自动切换。
  - challenge 类型扩展（如 PoRep/PoSt 新证明类型）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase8.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase8.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 新增 CLI 参数（草案）
```text
--reward-distfs-adaptive-multiplier-hash-mismatch <u32, >=1>
--reward-distfs-adaptive-multiplier-missing-sample <u32, >=1>
--reward-distfs-adaptive-multiplier-timeout <u32, >=1>
--reward-distfs-adaptive-multiplier-read-io-error <u32, >=1>
--reward-distfs-adaptive-multiplier-signature-invalid <u32, >=1>
--reward-distfs-adaptive-multiplier-unknown <u32, >=1>
```

### 配置结构（延续）
```rust
DistfsProbeRuntimeConfig {
  max_sample_bytes: u32,
  challenges_per_tick: u32,
  challenge_ttl_ms: i64,
  allowed_clock_skew_ms: i64,
  adaptive_policy: StorageChallengeAdaptivePolicy,
}
```

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH8-M1**：文档与任务拆解完成。
  - **DPH8-M2**：CLI multiplier 参数接线完成。
  - **DPH8-M3**：测试与可观测覆盖完成。
  - **DPH8-M4**：回归与文档收口完成。
- Technical Risks:
  - 参数数量继续增加，运维复杂度上升；通过默认值与严格参数校验控制风险。
  - 倍率配置过高可能带来过度退避；通过 `backoff_max_ms` 上限兜底。
  - 主入口帮助文本持续膨胀；通过解析逻辑模块化和测试覆盖控制维护成本。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-074-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-074-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
