# P2P 文档集中治理与过期归档

## 1. Executive Summary
- Problem Statement: 将仓库内分散的 P2P / 区块链 / 共识 / 分布式文件系统（DistFS）文档集中到 `doc/p2p`，降低检索和维护成本。
- Proposed Solution: 对明确已被后续阶段替代或历史分卷的文档进行归档，统一放置到 `doc/p2p/archive`。
- Success Criteria:
  - SC-1: 为归档文档增加“已过期”标识，避免误用旧方案。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want P2P 文档集中治理与过期归档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 扫描 `doc/` 下与以下主题强相关文档：
  - AC-2: `p2p`
  - AC-3: `blockchain`（区块链）
  - AC-4: `consensus`（共识）
  - AC-5: `distfs`（分布式文件系统）
  - AC-6: `libp2p` / `dht` / `gossip`
- Non-Goals:
  - 修改文档中描述的技术方案本身。
  - 重写历史内容，仅补充必要的过期标识。
  - 调整 `third_party/` 下任何文件。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/p2p-doc-consolidation.prd.md`
  - `doc/p2p/archive/p2p-doc-consolidation.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- 新目录：`doc/p2p/`
- 归档目录：`doc/p2p/archive/`
- 过期标识模板（加在归档文档开头）：
  - `> [!WARNING]`
  - `> 该文档已过期，仅供历史追溯，不再作为当前实现依据。`
  - `> 归档日期：2026-02-16`
- 迁移清单文件：`doc/p2p/archive/migration-map.md`

#### 二轮核查结论（2026-02-16）
- 通过文档路径引用存在性检查，发现一批文档仍依赖已删除路径：
  - `crates/agent_world/src/runtime/distributed*`
  - `crates/agent_world/src/runtime/distributed_membership_sync*`
- 上述能力已迁移到 split crate（`agent_world_consensus` / `agent_world_net` / `agent_world_distfs`），原文档技术落点已过期。
- 已将该批文档归档到 `doc/p2p/archive`，并统一补充“已过期”标识。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：建立治理文档（设计 + 项目管理）并冻结判定口径。
  - M2：完成文档迁移到 `doc/p2p`。
  - M3：完成过期文档归档并补充过期标识。
  - M4：完成路径引用修复与基础校验。
  - M5：完成二轮“文档-代码”对照核查，追加归档技术口径过期文档。
- Technical Risks:
  - 批量迁移后可能存在遗漏引用，导致文档链接断裂。
  - “过期”判定可能存在主观性，需要采用“明确历史分卷/明确被后续阶段替代”作为保守标准。
  - 文档数量较多，需通过迁移映射清单保证可追踪性。
  - 二轮归档后，部分活跃文档依赖关系会指向 archive，需要在后续新设计文档中逐步补齐“当前基线文档”。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-043-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-043-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
