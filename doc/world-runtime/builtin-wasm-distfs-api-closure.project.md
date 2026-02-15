# World Runtime：Builtin Wasm DistFS API 闭环（项目管理文档）

## 任务拆解
- [x] DAC-1 输出设计文档（`doc/world-runtime/builtin-wasm-distfs-api-closure.md`）与项目管理文档（本文件）。
- [ ] DAC-2 扩展 `agent_world_distfs`：支持可选 hash 算法（sha256）并补充测试。
- [ ] DAC-3 新增 DistFS hydrate 工具并接入 `sync-m1/m4` 脚本写入路径。
- [ ] DAC-4 runtime builtin wasm 读取切到 `agent_world_distfs` API。
- [ ] DAC-5 执行 required 回归并回写文档/devlog。

## 依赖
- `crates/agent_world_distfs/src/lib.rs`
- `crates/agent_world_distfs/src/bin/`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`
- `crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/m4_builtin_wasm_artifact.rs`

## 状态
- 当前阶段：DAC-1 已完成，进入 DAC-2。
- 最近更新：新增 DistFS API 闭环改造任务（2026-02-15）。
