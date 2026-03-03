> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：删除旧 `agent_world_builtin_wasm` 并替换为新流程设计

## 1. Executive Summary
- 删除旧 `crates/agent_world_builtin_wasm`（名称与路径），避免继续把其作为“旧流程核心”。
- 以新的生命周期流程替代：`module crate -> agent_world_wasm_sdk lifecycle trait -> 新 runtime core crate`。
- 保持外部 ABI 与构建链路稳定（`alloc/reduce/call`、`module_id`、hash/DistFS 同步流程不变）。

## 2. User Experience & Functionality
### In Scope
- 将旧 crate 迁移为新命名 runtime core crate（承载 builtin 模块共享逻辑与常量导出）。
- 全量替换引用：
  - `agent_world` runtime 常量 re-export；
  - 23 个 builtin wasm module crate 的依赖与调用路径；
  - workspace 成员与锁文件。
- 更新构建元信息中 crate 标识（bootstrap manifest 字符串中的 crate 字段）。
- 保持 `scripts/sync-m1/m4-builtin-wasm-artifacts.sh` 与现有 hash/DistFS 闭环可用。

### Out of Scope
- 不改 builtin 模块业务逻辑（规则/经济/记忆/身体等语义不变）。
- 不改 wasm ABI 协议和 router/executor 行为。
- 不改 third_party 代码。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- 新 runtime core crate（替代旧 `agent_world_builtin_wasm`）：
  - 导出模块 ID/版本常量；
  - 导出共享执行入口（供模块 crate 的 lifecycle 实现调用）。
- 23 个模块 crate：
  - 继续通过 `agent_world_wasm_sdk::WasmModuleLifecycle` + `export_wasm_module!` 导出。
  - 依赖从旧 crate 切换到新 runtime core crate。
- bootstrap build manifest 字符串：
  - 从 `crate=agent_world_builtin_wasm` 切换为新 crate 名称，保证元信息一致性。

## 5. Risks & Roadmap
- RCR-1 文档任务：设计文档与项目管理文档落地。
- RCR-2 代码任务：删除旧 crate、替换引用、跑构建同步与回归验证。

### Technical Risks
- 风险：引用遗漏导致编译失败。
  - 缓解：全仓 `rg` 检查 + `cargo check`。
- 风险：模块 lock/hash 漂移。
  - 缓解：执行 `sync-m1/m4` 与 `--check`。
- 风险：bootstrap 元信息历史对比出现差异。
  - 缓解：仅更新 crate 字段，保持其余字段不变并记录到 devlog。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-ENGINEERING-006 | 文档内既有任务条目 | `test_tier_required` | `./scripts/doc-governance-check.sh` + 引用可达性扫描 | 迁移文档命名一致性与可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-DOC-MIG-20260303 | 逐篇阅读后人工重写为 `.prd` 命名 | 仅批量重命名 | 保证语义保真与审计可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章 Executive Summary。
- 原“范围” -> 第 2 章 User Experience & Functionality。
- 原“接口 / 数据” -> 第 4 章 Technical Specifications。
- 原“里程碑/风险” -> 第 5 章 Risks & Roadmap。
