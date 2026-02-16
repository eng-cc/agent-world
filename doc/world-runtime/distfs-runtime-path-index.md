# Agent World Runtime：DistFS 路径索引接入 execution_storage（设计文档）

## 目标
- 将 `agent_world_distfs::FileStore` 接入 `agent_world_net::execution_storage`，为执行产物提供稳定路径索引。
- 保持 CAS（content hash）作为底层真相，路径索引只作为可读、可枚举入口。
- 提供最小闭环：执行结果写入时生成路径索引，后续可按 `world_id + height` 回读区块与最新 head。

## 范围

### In Scope
- 在 `agent_world_net` 增加“写执行结果 + 写路径索引”的组合接口。
- 约定执行产物路径布局（`worlds/<world_id>/...`）。
- 提供路径读取接口（按高度读 block、读最新 head）。
- 补齐单元测试（写入、回读、非法 world_id）。

### Out of Scope
- 跨节点目录同步与冲突解决。
- 路径索引的版本迁移策略。
- 目录分页、GC 与冷热分层策略。

## 接口 / 数据

### 新增接口（草案）
- `store_execution_result_with_path_index(...)`：
  - 先复用现有 `store_execution_result` 写 CAS
  - 再写路径索引文件
- `load_block_by_height_from_path_index(world_id, height, store)`：
  - 从路径索引读取 `WorldBlock`
- `load_latest_head_from_path_index(world_id, store)`：
  - 从路径索引读取 `WorldHeadAnnounce`

### 路径约定（草案）
- `worlds/<world_id>/heads/latest_head.cbor`
- `worlds/<world_id>/blocks/<height_20>/block.cbor`
- `worlds/<world_id>/blocks/<height_20>/snapshot_manifest.cbor`
- `worlds/<world_id>/blocks/<height_20>/journal_segments.cbor`

说明：
- `<height_20>` 使用 20 位零填充十进制，保证字典序与高度序一致。
- `world_id` 作为路径分段使用，需进行分段安全校验（限制字符集）。

## 里程碑
- DPRI-1：设计文档与项目管理文档落地。
- DPRI-2：execution_storage 路径索引写入/读取实现。
- DPRI-3：单元测试与 crate 级回归。
- DPRI-4：状态文档与 devlog 收口。

## 风险
- `world_id` 兼容性：历史 world_id 若含非法路径字符，会被拒绝写入路径索引。
- 双写一致性：CAS 写成功但路径索引写失败时，存在“可验证但不可按路径检索”的窗口。
- 路径布局固定后，后续若需迁移需引入版本化策略。
