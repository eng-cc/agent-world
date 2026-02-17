# Agent World Runtime：Builtin WASM 模块独立工程化（MIP）设计文档

## 目标
- 将 builtin wasm 从“单 crate 多模块分发”迁移为“一个 `module_id` 对应一个独立 Rust 工程（crate）”。
- 保持运行时接口不变：runtime 继续按 `module_id` + hash 清单装载 wasm 工件。
- 构建链路改为 `module_id -> manifest_path` 显式映射，避免同一 manifest 重复构建导致所有模块字节一致。

## 范围

### In Scope
- 把 `crates/agent_world_builtin_wasm` 收敛为共享逻辑库（不再作为统一 wasm 工件入口）。
- 为每个 M1/M4 builtin 模块创建独立 wasm crate（独立 `Cargo.toml` + `src/lib.rs` 导出 `alloc/reduce/call`）。
- 新增模块清单映射文件（`module_id -> manifest_path`），并改造 `scripts/build-builtin-wasm-modules.sh` 按映射构建。
- 兼容现有 `sync-m1-builtin-wasm-artifacts.sh` / `sync-m4-builtin-wasm-artifacts.sh`、hash 清单与 DistFS hydration 流程。

### Out of Scope
- 不改变模块业务语义（规则、身体、记忆、电力、M4 经济逻辑保持不变）。
- 不重构 runtime 装载协议与 ABI（仍沿用 `wasm-1`、`alloc/reduce/call`）。
- 不在本轮引入额外构建后端（仅 Rust/wasm32）。

## 接口 / 数据
- 共享逻辑库：
  - `crates/agent_world_builtin_wasm`
  - 提供：`builtin_alloc`、`reduce_for_module(module_id, ptr, len)`、`call_for_module(module_id, ptr, len)`
- 独立模块工程（新增）：
  - `crates/agent_world_builtin_wasm_modules/<module_crate>/Cargo.toml`
  - `crates/agent_world_builtin_wasm_modules/<module_crate>/src/lib.rs`
  - 每个模块 crate 只绑定一个固定 `module_id`。
- 构建映射（新增）：
  - `crates/agent_world/src/runtime/world/artifacts/builtin_module_manifest_map.txt`
  - 格式：`<module_id><空格><manifest_path>`
- 构建脚本改造：
  - `scripts/build-builtin-wasm-modules.sh` 新增映射解析。

## 里程碑
- MIP-1：设计文档与项目管理文档落地。
- MIP-2：完成模块独立工程拆分与构建链路改造。
- MIP-3：完成 hash/DistFS/回归收口并更新文档状态。

## 风险
- 映射文件与模块 ID 清单漂移：通过脚本构建时强校验（缺失即失败）。
- 模块 crate 数量增加导致维护成本上升：用统一模板与映射文件降低重复维护面。
- 迁移期构建失败风险：保留最小可回归命令并在收口任务执行全链路检查。
