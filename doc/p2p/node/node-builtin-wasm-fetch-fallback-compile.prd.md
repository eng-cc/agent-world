# World Runtime：Builtin Wasm 先拉取后编译回退

- 对应设计文档: `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.design.md`
- 对应项目管理文档: `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 在节点运行时装载 builtin wasm 时，优先从网络拉取已构建产物；拉取失败后再触发本地编译回退。
- Proposed Solution: 保持去中心化：不引入中心化 Builder，任何节点都可以在本地编译并产出可校验 wasm。
- Success Criteria:
  - SC-1: 与既有模块身份校验对齐：最终落地的 wasm 必须匹配 `wasm_hash`，避免不同机器构建漂移导致的无声错误。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want World Runtime：Builtin Wasm 先拉取后编译回退 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: runtime builtin wasm 读取链路改造：`local distfs -> network fetch -> local compile fallback`。
  - AC-2: 增加可配置的 fetch / compiler 接口（通过环境变量注入），便于不同部署接入各自网络层。
  - AC-3: 编译回退结果写入本地 distfs（sha256），并立即复用。
  - AC-4: 增加 `test_tier_full` 闭环测试：覆盖“拉取失败 -> 本地编译 -> 成功装载并缓存”。
- Non-Goals:
  - 引入新的中心化构建服务。
  - 统一的全网 artifact 发布/治理协议（本轮仅在节点内落地 fetch+fallback 行为）。
  - 非 builtin wasm 模块的源码分发协议。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.prd.md`
  - `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- 运行时入口：
  - `crates/oasis7/src/runtime/m1_builtin_wasm_artifact.rs`
  - `crates/oasis7/src/runtime/m4_builtin_wasm_artifact.rs`
- 新增内置装载辅助：
  - 本地验证读取：`sha256` + distfs `get_verified`
  - 网络拉取（可配置）：通过 fetcher/URL 配置尝试获取 `<wasm_hash>` 对应 bytes
  - 回退编译（可配置）：按 `module_id` 在节点本地编译 wasm，写回 distfs
- 关键环境变量（设计约定）：
  - `OASIS7_BUILTIN_WASM_FETCHER`
  - `OASIS7_BUILTIN_WASM_FETCH_URLS`
  - `OASIS7_BUILTIN_WASM_COMPILER`
  - `OASIS7_BUILTIN_WASM_DISTFS_ROOT`

## 5. Risks & Roadmap
- Phased Rollout:
  - **BFC-1**：设计文档与项目管理文档落地。
  - **BFC-2**：runtime builtin wasm 拉取/回退编译链路实现。
  - **BFC-3**：`test_tier_full` 闭环测试落地。
  - **BFC-4**：回归、项目文档状态更新、任务日志收口。
- Technical Risks:
  - 本地编译回退依赖节点编译环境（toolchain/target）可用；需清晰报错。
  - 远端返回错误产物时必须严格 hash 校验，避免污染本地缓存。
  - 测试若直接依赖真实网络可能不稳定，需使用可控闭环场景。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-087-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-087-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
