# Agent World Runtime：PoS 历史设计文档归档收敛（2026-02-20）

## 1. Executive Summary
- Problem Statement: 将已经完成且被后续实现替代的早期 Node PoS 分阶段设计文档归档到 `doc/p2p/archive`。
- Proposed Solution: 保留历史追溯能力，同时避免团队把阶段性文档当作当前实现基线。
- Success Criteria:
  - SC-1: 保持 `doc/p2p` 活跃目录中的 PoS 文档口径与当前代码一致。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：PoS 历史设计文档归档收敛（2026-02-20） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 归档以下 3 组 PoS 早期文档及其项目管理文档：
  - AC-2: `distributed-node-mainloop*`
  - AC-3: `distributed-node-pos-mainloop*`
  - AC-4: `distributed-node-pos-gossip*`
  - AC-5: 在归档文档文件头补充过期警示块与归档日期。
  - AC-6: 修复仓库内对上述文档的路径引用（仅路径层面，不重写历史内容）。
- Non-Goals:
  - 修改任何运行时代码或共识语义。
  - 重写历史方案细节或变更原任务结论。
  - 大规模重构其他主题文档。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/pos-doc-archive-cleanup-2026-02-20.prd.md`
  - `doc/p2p/archive/pos-doc-archive-cleanup-2026-02-20.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- 归档目标目录：`doc/p2p/archive/`
- 归档警示模板：
  - `> [!WARNING]`
  - `> 该文档已过期，仅供历史追溯，不再作为当前实现依据。`
  - `> 归档日期：2026-02-20`
- 当前 PoS 基线文档：
  - `doc/p2p/consensus/consensus-code-consolidation-to-agent-world-consensus.prd.md`
  - `doc/p2p/consensus/consensus-code-consolidation-to-agent-world-consensus.prd.project.md`

## 5. Risks & Roadmap
- Phased Rollout:
  - PDA-1：新增归档收口设计/项目管理文档。
  - PDA-2：完成 3 组历史 PoS 文档迁移到 `doc/p2p/archive` 并加过期警示。
  - PDA-3：完成引用路径修复、回归检查、devlog 收口与提交。
- Technical Risks:
  - 若遗漏引用修复，会产生活跃文档断链。
  - 若误归档仍在迭代中的文档，会影响当前设计基线。
  - 归档后需确保团队入口文档仍指向当前有效基线。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-044-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-044-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
