# Agent World Runtime：分布式能力彻底拆分（Phase 7）

## 1. Executive Summary
- Problem Statement: 完成分布式能力从 `agent_world` 到基础 crate 的彻底拆分：`agent_world_net` / `agent_world_consensus` / `agent_world_distfs` / `agent_world_proto`。
- Proposed Solution: 删除 `agent_world` 内分布式实现文件，`agent_world` 仅保留世界内核与模拟层，不再承载分布式实现逻辑。
- Success Criteria:
  - SC-1: 将 Viewer 协议并入 `agent_world_proto`，消除 `agent_world` 对 Viewer 协议层的承载。
  - SC-2: 收敛 WASM ABI 边界，消除 net 侧重复清单定义，明确 ABI 与运行时缓存归属。
  - SC-3: 对超 1200 行 Rust 文件完成物理拆分，满足仓库维护约束。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：分布式能力彻底拆分（Phase 7） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增 `agent_world_distfs` crate，并迁移分布式文件能力：CAS、分片、组装校验。
  - AC-2: 分布式彻底拆分：删除 `crates/agent_world/src/runtime/distributed*`、`libp2p_net.rs` 及相关分布式路径代码。
  - AC-3: `agent_world` facade 收敛：移除分布式导出，按领域划分导出边界。
  - AC-4: Viewer 协议迁移到 `agent_world_proto` 并完成 server/viewer 双端适配。
  - AC-5: WASM ABI 边界调整：消除 `agent_world_net` 内重复 `ModuleManifest`，统一到 ABI/proto。
  - AC-6: 超长文件拆分（>1200 行）。
- Non-Goals:
  - 新增分布式功能语义（本阶段只做拆分与归位，不扩展新协议能力）。
  - 引入新的传输协议或更换现有网络后端。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distributed/distributed-hard-split-phase7.prd.md`
  - `doc/p2p/distributed/distributed-hard-split-phase7.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- `agent_world_distfs`：
  - `BlobStore` / `LocalCasStore`
  - `segment_snapshot` / `segment_journal`
  - `assemble_snapshot` / `assemble_journal`
- `agent_world_proto`：
  - 新增 Viewer 协议模块（请求/响应/控制字段）
  - 继续作为跨 crate 协议与错误模型载体
- `agent_world_wasm_abi`：
  - 统一模块清单类型来源，去掉 net 侧重复结构

## 5. Risks & Roadmap
- Phased Rollout:
  - M13-1：`agent_world_distfs` 落地并接入现有路径
  - M13-2：`agent_world` 分布式文件删除完成，调用方切换到基础 crate
  - M13-3：`agent_world` facade 收敛完成
  - M13-4：Viewer 协议并入 proto 并适配完成
  - M13-5：WASM ABI 边界收敛完成
  - M13-6：超长文件拆分与回归完成
- Technical Risks:
  - 大规模路径调整会导致编译面回归；需按任务分批编译与定向测试。
  - API 导出收敛会影响现有调用方；需同步修改 workspace 内依赖点。
  - Viewer 协议迁移会带来序列化兼容风险；需保证 server/viewer 协议版本一致。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-081-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-081-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
