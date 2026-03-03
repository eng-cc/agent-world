# Agent World Runtime：DistFS 标准文件读写接口（设计文档）

## 目标
- 在 `agent_world_distfs` 内提供最小可用的“标准文件读写接口”，补齐当前仅有 blob/CAS 接口的缺口。
- 保持内容寻址（CAS）为底层真相，文件路径只是到 `content_hash` 的可变索引映射。
- 提供本地可测试闭环：写文件、读文件、列目录、查询元信息、删除映射。

## 范围

### In Scope
- 在 `agent_world_distfs` 增加文件抽象接口（path -> content_hash）。
- 本地索引存储（JSON）及原子写入。
- 路径校验（禁止空路径、`..` 穿越、绝对路径）。
- 基础单元测试：读写回读、覆盖写、删除、路径校验。

### Out of Scope
- 跨节点分布式复制一致性。
- 文件权限模型、ACL、租约写锁。
- 目录树高阶操作（move/copy/glob/watch）。
- 加密存储与内容访问审计。

## 接口 / 数据

### 文件接口（草案）
- `FileStore::write_file(path, bytes)`：写入文件路径，返回文件元信息。
- `FileStore::read_file(path)`：按路径读取内容。
- `FileStore::delete_file(path)`：删除路径映射（不强制删除底层 blob）。
- `FileStore::stat_file(path)`：读取路径对应元信息。
- `FileStore::list_files()`：列出已登记路径。

### 元信息模型（草案）
```rust
FileMetadata {
  path: String,
  content_hash: String,
  size_bytes: u64,
  updated_at_ms: i64,
}
```

### 本地索引文件（草案）
- 文件名：`files_index.json`
- 结构：
```rust
FileIndexFile {
  version: u64,
  files: BTreeMap<String, FileMetadata>,
}
```
- 版本：`version = 1`

### 语义约束
- 写入文件时：
  - 先写入底层 CAS（`put_bytes`）
  - 再更新路径索引
- 删除文件时：
  - 仅删除路径映射
  - 底层 blob 生命周期由 pin/evict/GC 决策

## 里程碑
- DFIO-1：设计文档与项目管理文档。
- DFIO-2：文件接口与本地索引实现。
- DFIO-3：单元测试与回归。

## 风险
- 文件路径到 hash 的索引可能与外部期望不一致（如覆盖语义），需通过接口文档明确。
- 长期运行下索引文件可能增长，需要后续分页或分片。
- 仅本地语义尚未覆盖跨节点并发写冲突。
