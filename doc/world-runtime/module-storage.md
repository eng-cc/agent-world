# Agent World Runtime：模块存储持久化（设计文档）

## 目标
- 提供模块工件（WASM bytes）与模块注册表（registry）的**可持久化存储**。
- 支持冷启动加载模块注册表与工件元信息，保证回放与治理闭环一致性。
- 提供可测试、可替换的本地文件存储实现（未来可替换为 KV/对象存储）。

## 范围

### In Scope
- 本地目录结构与文件格式约定（registry/meta/artifacts）。
- `ModuleStore` 读写 API（写入工件、读写注册表、读写 meta）。
- 版本号校验与基础错误处理。

### Out of Scope
- 远端对象存储、分布式一致性、加密存储。
- 大规模分片、增量同步与压缩。

## 接口 / 数据

### 存储布局（默认）
- `module_registry.json`：模块索引与活动版本
- `modules/<wasm_hash>.wasm`：模块工件（只读）
- `modules/<wasm_hash>.meta.json`：模块元信息（ModuleManifest 快照）

### 关键数据结构（示意）
```rust
struct ModuleRegistryFile {
  version: u64,
  updated_at: i64,
  records: BTreeMap<String, ModuleRecord>,
  active: BTreeMap<String, String>,
}
```

### ModuleStore API（示意）
- `ModuleStore::new(root)`：基于根目录构造存储
- `write_artifact(wasm_hash, bytes)`：写入工件
- `read_artifact(wasm_hash)`：读取工件
- `write_meta(manifest)`：写入 meta.json
- `read_meta(wasm_hash)`：读取 meta.json
- `save_registry(registry)`：保存注册表
- `load_registry()`：读取注册表并校验版本

### 版本策略
- `module_registry.json` 使用 `version=1`，不支持版本直接拒绝加载。

## 里程碑
- **S1**：实现本地文件存储 ModuleStore
- **S2**：接入 world 保存/加载流程（可选）

## 风险
- 文件写入中断导致部分写入，需要原子写入策略。
- 工件体积大，读写影响性能。
- 版本升级时兼容性处理复杂。
