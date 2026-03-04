# world-runtime PRD

审计轮次: 1

## 目标
- 建立 world-runtime 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 world-runtime 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 world-runtime 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/world-runtime/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/world-runtime/prd.md`
- 项目管理入口: `doc/world-runtime/prd.project.md`
- 文件级索引: doc/world-runtime/prd.index.md
- 追踪主键: `PRD-WORLD_RUNTIME-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: world runtime 涉及确定性执行、事件溯源、WASM 扩展、治理与审计等核心能力，若缺少统一设计入口，跨阶段改动容易引发一致性与安全回归。
- Proposed Solution: 以 world-runtime PRD 统一定义内核能力边界、WASM 运行约束、治理流程、数值语义与验证标准。
- Success Criteria:
  - SC-1: runtime 关键改动具备 PRD-WORLD_RUNTIME-ID 映射与测试证据。
  - SC-2: 确定性回放与事件审计链路保持可复现。
  - SC-3: WASM 沙箱与接口变更具备兼容性与安全校验记录。
  - SC-4: 数值语义硬化议题持续收敛并形成阶段性里程碑。

## 2. User Experience & Functionality
- User Personas:
  - 运行时架构师：需要控制可信边界与模块化演进。
  - 模块开发者：需要稳定 ABI/执行语义与治理流程。
  - 审计与安全评审者：需要完整可追溯的事件与收据链路。
- User Scenarios & Frequency:
  - 运行时语义评审：每次核心行为改动前执行，确认确定性与兼容边界。
  - WASM 接口变更：每个接口变更至少进行一次兼容核验与回放验证。
  - 治理事件审计：发布前执行，检查关键治理事件链路完整性。
  - 安全回归复核：按周执行，验证沙箱、签名、权限约束无回退。
- User Stories:
  - PRD-WORLD_RUNTIME-001: As a 架构师, I want deterministic world execution semantics, so that replay and audit remain trustworthy.
  - PRD-WORLD_RUNTIME-002: As a 模块开发者, I want stable WASM interfaces and lifecycle governance, so that upgrades are safe.
  - PRD-WORLD_RUNTIME-003: As a 安全评审者, I want explicit security and receipt guarantees, so that critical risks are controlled.
- Critical User Flows:
  1. Flow-WR-001: `提交 runtime 变更 -> 执行回放一致性验证 -> 对比事件链 -> 输出兼容结论`
  2. Flow-WR-002: `WASM 模块注册/升级 -> 生命周期治理校验 -> 沙箱执行 -> 审计事件归档`
  3. Flow-WR-003: `安全异常发现 -> 回溯 receipt -> 定位策略缺口 -> 补回归与发布阻断`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 确定性执行与回放 | action/event 序列、snapshot、replay 差异 | 执行回放并比较关键状态 | `pending -> replaying -> matched/mismatched` | 按 tick 与 event id 有序比较 | 仅运行时维护者可调整基线 |
| WASM 生命周期治理 | 模块ID、版本、哈希、策略 | 注册/升级/停用流程带审计输出 | `register -> activate -> deactivate -> upgrade`（治理动作） | 版本与策略双约束 | 未授权模块不得激活 |
| 审计与收据链路 | effect、receipt、签名、cause | 导出审计记录并验证签名 | `emitted -> signed -> verified/rejected` | 按事件时间与重要级别检索 | 安全评审者可查看完整链路 |
- Acceptance Criteria:
  - AC-1: world-runtime PRD 覆盖内核、WASM、治理、安全四条主线。
  - AC-2: world-runtime project 文档任务映射 PRD-ID 并维护状态。
  - AC-3: 与 `doc/world-runtime/runtime/runtime-integration.md`、`doc/world-runtime/wasm/wasm-interface.md` 等分册一致。
  - AC-4: 关键行为变更同步更新测试方案与执行记录。
- Non-Goals:
  - 不在本 PRD 中展开每个阶段的实现代码细节。
  - 不替代 p2p 网络拓扑或 site 发布策略设计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: WASM 执行与治理测试、审计导出工具、数值语义回归套件。
- Evaluation Strategy: 以回放一致性、治理事件完整度、沙箱安全回归数、数值语义缺陷收敛率评估。

## 4. Technical Specifications
- Architecture Overview: world-runtime 模块是系统可信执行基座，负责世界状态演化、模块扩展执行与治理审计，向上游 simulator/game 与下游 p2p 提供稳定语义。
- Integration Points:
  - `doc/world-runtime/runtime/runtime-integration.md`
  - `doc/world-runtime/wasm/wasm-interface.md`
  - `doc/world-runtime/wasm/wasm-executor.prd.md`
  - `doc/world-runtime/governance/governance-events.md`
  - `doc/world-runtime/testing/testing.md`
- Edge Cases & Error Handling:
  - 回放不一致：立即标记高风险阻断并输出差异快照。
  - 接口超时/失败：WASM 执行异常需返回结构化错误而非 panic。
  - 空事件流：空输入需稳定返回，无副作用写入。
  - 权限不足：未授权模块请求直接拒绝并记录审计事件。
  - 并发冲突：治理操作并发时按版本序列化处理，拒绝乱序变更。
  - 数据异常：receipt 校验失败时不得推进状态并触发安全告警。
- Non-Functional Requirements:
  - NFR-WR-1: 同一输入回放结果一致率 100%。
  - NFR-WR-2: 关键治理事件审计链路完整率 100%。
  - NFR-WR-3: WASM 接口变更需保持向后兼容或明确破坏性声明。
  - NFR-WR-4: 安全相关回归在 full 层级覆盖率达到目标阈值并持续跟踪。
  - NFR-WR-5: 核心运行时异常可在 30 分钟内完成初步定位。
- Security & Privacy: 强制最小权限、签名校验、审计留痕；禁止未授权模块绕过规则层直接修改世界状态。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化 runtime 模块设计基线与边界约束。
  - v1.1: 补齐 WASM 生命周期与治理流程的跨模块验收清单。
  - v2.0: 建立运行时质量趋势报表（确定性、安全、性能、治理事件完整度）。
- Technical Risks:
  - 风险-1: 运行时复杂度提升导致验证成本增加。
  - 风险-2: ABI/治理策略变更引发兼容性断裂。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-WORLD_RUNTIME-001 | TASK-WORLD_RUNTIME-001/002/005 | `test_tier_required` + `test_tier_full` | 回放一致性、核心边界验收清单校验 | 世界状态演化与确定性语义 |
| PRD-WORLD_RUNTIME-002 | TASK-WORLD_RUNTIME-002/003/005 | `test_tier_required` | WASM 接口兼容性检查、治理流程测试 | 模块升级与生命周期稳定性 |
| PRD-WORLD_RUNTIME-003 | TASK-WORLD_RUNTIME-003/004/005 | `test_tier_full` | 收据签名校验、安全回归抽样 | 审计可信性与安全边界 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-WR-001 | 以确定性回放作为运行时验收核心 | 仅执行结果正确即可 | 可追溯和审计需求要求强确定性。 |
| DEC-WR-002 | WASM 生命周期走治理流程 | 模块直接热替换 | 无治理热替换难以保证安全与一致性。 |
| DEC-WR-003 | 安全事件必须输出可验证 receipt | 仅日志文本记录 | 签名收据可支撑事后审计与取证。 |
