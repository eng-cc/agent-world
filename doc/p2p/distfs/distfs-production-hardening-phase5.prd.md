# oasis7 Runtime：DistFS 生产化增强（Phase 5）设计文档

- 对应设计文档: `doc/p2p/distfs/distfs-production-hardening-phase5.design.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase5.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口文档：`doc/p2p/distfs/distfs-production-hardening-phase1.prd.md`。
- 本文件为 Phase 5 增量子文档（slave），仅维护本阶段增量内容。

## 1. Executive Summary
- Problem Statement: 把 DistFS challenge probe 从硬编码参数升级为运行时可配置参数，支持按网络状态调优挑战强度。
- Proposed Solution: 将 reward runtime 中 DistFS probe 逻辑模块化，降低主文件复杂度并保持单文件行数安全。
- Success Criteria:
  - SC-1: 增强挑战可观测性：在 epoch 报告中输出 probe 配置与累计 cursor 状态快照。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want oasis7 Runtime：DistFS 生产化增强（Phase 5）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **DPH5-1：文档与任务拆解**
  - AC-2: 输出 Phase 5 设计文档与项目管理文档。
  - AC-3: **DPH5-2：probe 参数治理化与模块化拆分**
  - AC-4: 新增 DistFS probe runtime 子模块，承载：
  - AC-5: probe 配置结构与默认值；
  - AC-6: probe 状态加载/持久化；
- Non-Goals:
  - 链上治理参数动态下发。
  - 跨进程集中式 challenge coordinator。
  - PoRep/PoSt 协议级升级。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-production-hardening-phase5.prd.md`
  - `doc/p2p/distfs/distfs-production-hardening-phase5.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### CLI 参数（草案）
```text
--reward-distfs-probe-max-sample-bytes <u32, >0>
--reward-distfs-probe-per-tick <u32, >0>
--reward-distfs-probe-ttl-ms <i64, >0>
--reward-distfs-probe-allowed-clock-skew-ms <i64, >=0>
```

### runtime 配置（草案）
```rust
DistfsProbeRuntimeConfig {
  max_sample_bytes: u32,
  challenges_per_tick: u32,
  challenge_ttl_ms: i64,
  allowed_clock_skew_ms: i64,
}
```

### 报告新增字段（草案）
```json
{
  "distfs_probe_config": { ... },
  "distfs_probe_cursor_state": { ... },
  "distfs_challenge_report": { ... }
}
```

## 5. Risks & Roadmap
- Phased Rollout:
  - **DPH5-M1**：文档与任务拆解完成。
  - **DPH5-M2**：参数治理化 + 模块化拆分完成。
  - **DPH5-M3**：报告可观测增强完成。
  - **DPH5-M4**：回归与文档收口完成。
- Technical Risks:
  - 参数暴露后配置错误风险上升；通过强校验与默认值兜底控制。
  - 模块拆分若边界不清会导致重复逻辑；通过单一入口函数约束。
  - 报告字段增多会增加 I/O 体积；当前字段规模可接受。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-071-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-071-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
