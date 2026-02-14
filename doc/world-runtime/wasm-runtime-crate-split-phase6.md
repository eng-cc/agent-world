# Agent World Runtime：WASM 运行时激进迁移（设计文档，Phase 6）

## 目标
- 继续收敛 `agent_world` 主 crate 的 WASM 运行时职责，将模块文件存储实现从 runtime 中抽离到独立 crate。
- 为后续 runtime 最小化与跨 crate 复用做准备，让 `agent_world` 保留编排与兼容层职责。

## 范围

### In Scope（本次）
- 新增独立 crate（暂定 `agent_world_wasm_store`），承载模块工件/元数据/注册表的文件持久化实现。
- 将现有 `ModuleStore` 核心文件读写逻辑迁移到新 crate。
- `agent_world` 侧保留 `ModuleStore` 对外 API（可通过轻量包装 + 错误映射维持兼容）。
- 回归验证 `module_store` 与 world 持久化相关测试路径。

### Out of Scope（本次不做）
- 改动模块存储目录结构（`module_registry.json` + `modules/*.wasm` + `*.meta.json` 保持不变）。
- 修改治理流程、模块注册语义或事件 schema。
- 引入远端对象存储/分布式存储后端。

## 接口 / 数据
- 新 crate 依赖 `agent_world_wasm_abi` 复用 `ModuleManifest/ModuleRegistry/ModuleRecord`。
- `agent_world` 根导出继续暴露 `ModuleStore`，尽量不影响上层调用方。
- 错误语义保持一致（版本不匹配、I/O/序列化失败）。

## 里程碑
- **R6-0**：文档与任务拆解完成。
- **R6-1**：`ModuleStore` 文件存储实现拆到独立 crate 并回归通过。

## 风险
- 错误映射若遗漏，可能导致现有测试断言和上层错误处理行为变化。
- 抽离后若路径/原子写细节偏移，可能影响模块工件持久化稳定性。
