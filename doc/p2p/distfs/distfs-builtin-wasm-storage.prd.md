# World Runtime：Builtin Wasm DistFS 存储与提交前校验

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 将内置 builtin wasm 二进制从 git 跟踪中移出，避免仓库长期携带二进制大文件。
- Proposed Solution: 保留并强化可追踪的一致性基线：`module_id -> wasm_hash` 清单继续由 git 跟踪。
- Success Criteria:
  - SC-1: 将 wasm 校验前移到提交前（pre-commit）执行，尽早拦截“源码已变更但 hash 未更新”问题。
  - SC-2: 运行时装载路径从 `include_bytes!` 切换为 DistFS 本地存储读取，并做 hash 校验。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want World Runtime：Builtin Wasm DistFS 存储与提交前校验 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: In Scope：
  - AC-2: `scripts/sync-m1-builtin-wasm-artifacts.sh` / `scripts/sync-m4-builtin-wasm-artifacts.sh` 改造为“构建 + hash 校验 + DistFS 落盘”。
  - AC-3: `scripts/pre-commit.sh` 增加 builtin wasm 一致性校验步骤。
  - AC-4: runtime builtin artifact 加载逻辑改造：按 hash 清单从 DistFS 本地目录读取。
  - AC-5: `.gitignore` 更新与历史 wasm 文件移出 git 追踪。
  - AC-6: Out of Scope：
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-builtin-wasm-storage.prd.md`
  - `doc/p2p/distfs/distfs-builtin-wasm-storage.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- git 跟踪：
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_module_ids.txt`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_module_ids.txt`
- git 不跟踪：
  - `.distfs/builtin_wasm/blobs/<sha256>.blob`
- 脚本行为：
  - `--check`：构建 wasm，校验 built hash 与 git 清单一致，并将产物写入 DistFS 本地存储（用于后续测试运行时读取）。
  - 默认（非 `--check`）：构建 wasm，刷新 hash 清单并写入 DistFS 本地存储。
- 运行时行为：
  - 通过模块 id 查询 hash 清单。
  - 从 DistFS 本地目录读取 `<sha256>.blob`。
  - 读取后再次计算 SHA-256，必须与清单一致。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：文档与任务拆解完成。
  - M2：脚本与 pre-commit 改造完成。
  - M3：runtime 加载改造完成，移除 git 跟踪 wasm 二进制。
  - M4：required tier 回归、文档与 devlog 收口。
- Technical Risks:
  - 本地未执行同步脚本时，运行时读取 DistFS 可能报缺失。
  - 运行时从文件读取替代静态嵌入后，错误从“编译期”变为“运行期”，需要清晰报错与提示。
  - `--check` 带副作用（写入 DistFS 本地目录），需在文档中明确这是预期行为。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-060-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-060-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
