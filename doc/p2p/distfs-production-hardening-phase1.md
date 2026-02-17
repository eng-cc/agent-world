# Agent World Runtime：DistFS 生产化增强（Phase 1）设计文档

## 目标
- 在不破坏现有 `BlobStore`/`FileStore` 接口兼容性的前提下，补齐 DistFS 的基础生产语义能力。
- 提供“可审计、可回收、可同步、可并发保护”的最小闭环，降低文件索引漂移和脏写风险。
- 保持实现集中在 `agent_world_distfs`，便于上层 `agent_world_net` / `agent_world_node` 直接复用。

## 范围

### In Scope
- **DPH1-1：条件写删（CAS 语义）**
  - 新增路径级 compare-and-set 写入能力，避免“无条件覆盖”导致的并发丢更新。
  - 新增路径级 compare-and-set 删除能力，避免“错误版本删除”。

- **DPH1-2：索引审计与孤儿块回收**
  - 新增文件索引审计报告，识别：
    - 索引引用但 blob 缺失；
    - pin 引用但 blob 缺失；
    - blob 存在但未被索引且未被 pin（孤儿）。
  - 新增“仅清理孤儿 blob”的回收入口，保证不误删被索引或 pinned 数据。

- **DPH1-3：文件索引 Manifest 导出/导入**
  - 支持把当前 `files_index` 导出为 manifest 并落入 CAS，供跨节点同步/恢复使用。
  - 支持从 manifest 导入索引（严格校验：路径规范、hash 合法、blob 存在且大小匹配）。

- **DPH1-4：测试闭环**
  - 为上述能力补齐 `agent_world_distfs` 单元测试与回归。

### Out of Scope
- 多写者冲突自动合并（CRDT/OT）。
- 跨节点复制协议重构与共识提交。
- ACL、租约分布式锁、端到端加密与审计追踪。

## 接口 / 数据

### 条件写删接口（草案）
```rust
LocalCasStore::write_file_if_match(
  path: &str,
  expected_content_hash: Option<&str>,
  bytes: &[u8]
) -> Result<FileMetadata, WorldError>

LocalCasStore::delete_file_if_match(
  path: &str,
  expected_content_hash: Option<&str>
) -> Result<bool, WorldError>
```

语义：
- `expected_content_hash = Some(hash)` 时，要求当前路径存在且 hash 一致，否则拒绝。
- `expected_content_hash = None` 时，不做版本前置校验，行为与当前写删一致。

### 审计报告（草案）
```rust
FileIndexAuditReport {
  total_indexed_files: usize,
  total_pins: usize,
  missing_file_blob_hashes: Vec<String>,
  dangling_pin_hashes: Vec<String>,
  orphan_blob_hashes: Vec<String>,
}
```

### Manifest（草案）
```rust
FileIndexManifest {
  version: u64,
  files: Vec<FileMetadata>,
}

FileIndexManifestRef {
  content_hash: String,
  size_bytes: u64,
}
```

导出流程：
- 读取 `files_index` -> 规范化排序 -> canonical CBOR -> 落入 CAS -> 返回 `FileIndexManifestRef`。

导入流程：
- 拉取并解码 manifest -> 全量校验 -> 原子替换本地 `files_index`。

## 里程碑
- **DPH1-M1**：设计文档与项目管理文档完成。
- **DPH1-M2**：条件写删能力与测试完成。
- **DPH1-M3**：审计与孤儿回收能力与测试完成。
- **DPH1-M4**：Manifest 导出导入与测试完成。
- **DPH1-M5**：回归、文档状态更新、devlog 收口。

## 风险
- 条件写删引入后，调用方若未处理冲突错误，可能出现“重试风暴”；需要上层按版本冲突做退避重试。
- Manifest 导入是索引级替换，若使用方误传 manifest 可能覆盖本地映射；通过严格校验和错误语义降低风险。
- 孤儿回收若识别逻辑错误可能误删数据；本期仅回收“未被索引且未被 pin”的集合，并补单测保护。
