# World Runtime：Builtin Wasm DistFS API 闭环

## 1. Executive Summary
- Problem Statement: 将 builtin wasm 的“写入/读取”统一走 `agent_world_distfs` API，避免旁路文件操作。
- Proposed Solution: 让后续 wasm 模块扩展复用同一 DistFS API 路径（脚本落盘、运行时读取、hash 校验语义一致）。
- Success Criteria:
  - SC-1: 保持当前治理边界：git 继续追踪 `module_id -> sha256` 清单，wasm 二进制仍不入 git。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want World Runtime：Builtin Wasm DistFS API 闭环 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: In Scope：
  - AC-2: 为 `agent_world_distfs::LocalCasStore` 增加可选 hash 算法策略（默认 `blake3`，新增 `sha256`）。
  - AC-3: 新增 DistFS 工具入口，用于按 hash manifest + built wasm 目录写入 blob（走 `LocalCasStore::put`）。
  - AC-4: `scripts/sync-m4-builtin-wasm-artifacts.sh` 改为调用 DistFS 工具完成写入。
  - AC-5: runtime builtin wasm 读取改为使用 `LocalCasStore` API（读取并按 sha256 校验）。
  - AC-6: Out of Scope：
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/builtin-wasm-distfs-api-closure.prd.md`
  - `doc/p2p/distfs/builtin-wasm-distfs-api-closure.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- DistFS API：
  - `LocalCasStore::new_with_hash_algorithm(root, HashAlgorithm)`
  - `HashAlgorithm::{Blake3,Sha256}`
  - `BlobStore::put/get/has` 在 `LocalCasStore` 上按实例 hash 策略工作。
- 工具入口：
  - `cargo run -p agent_world_distfs --bin hydrate_builtin_wasm -- --root <distfs-root> --manifest <sha256-manifest> --built-dir <wasm-dir>`
- 运行时：
  - `m1/m4` builtin wasm 读取改为 `LocalCasStore` API。
  - 缺失/校验失败仍返回 `ModuleChangeInvalid`，并包含定位信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：设计文档与项目管理文档。
  - M2：`agent_world_distfs` hash 策略扩展 + hydrate 工具。
  - M3：sync 脚本与 runtime 读取切到 DistFS API。
  - M4：required 回归 + 文档/devlog 收口。
- Technical Risks:
  - 脚本每次触发 `cargo run` 的耗时会增加；先保证正确性，后续可做工具常驻/缓存优化。
  - hash 策略扩展需保持默认 `blake3` 行为不变，避免影响现有 distributed 路径。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-059-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-059-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
