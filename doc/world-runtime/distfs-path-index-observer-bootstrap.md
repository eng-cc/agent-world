# Agent World Runtime：Observer/Bootstrap 路径索引读取接入（设计文档）

## 目标
- 将 DistFS 路径索引读取能力接入 `agent_world_net` 的 bootstrap/observer 调用链。
- 在“本地已持有执行产物”的场景下，支持不依赖网络拉取完成世界恢复。
- 保持现有网络路径不变，新增显式路径索引入口，避免影响既有行为。

## 范围

### In Scope
- 新增 bootstrap 入口：
  - `bootstrap_world_from_head_with_path_index`
  - `bootstrap_world_from_latest_path_index`
- 在 `HeadFollower` 增加路径索引同步入口（基于 head 队列选择后应用）。
- 在 `ObserverClient` 增加路径索引同步/报告/循环跟随入口。
- 单元测试覆盖：路径索引恢复成功、latest head 恢复成功。

### Out of Scope
- 自动 fallback 策略（网络失败后自动回退路径索引）。
- 多副本路径索引冲突仲裁。
- 路径索引 GC/回收策略。

## 接口 / 数据

### 新增 API（草案）
- `bootstrap_world_from_head_with_path_index(head, store)`：
  - 从路径索引读取 block
  - 从本地 CAS 读取 manifest/segments
  - 复用 `validate_head_update` 校验并构建 `World`
- `bootstrap_world_from_latest_path_index(world_id, store)`：
  - 读取 `latest_head.cbor`，再走 `bootstrap_world_from_head_with_path_index`

### 调用链接入（草案）
- `HeadFollower`：
  - `apply_head_with_path_index`
  - `sync_from_heads_with_path_index`
- `ObserverClient`：
  - `sync_heads_with_path_index`
  - `sync_heads_with_path_index_report`
  - `sync_heads_with_path_index_result`
  - `follow_heads_with_path_index`

## 里程碑
- POBI-1：设计文档与项目管理文档落地。
- POBI-2：bootstrap/head_follow/observer 路径索引入口实现。
- POBI-3：单元测试与 `agent_world_net` 回归。
- POBI-4：状态文档与 devlog 收口。

## 风险
- 路径索引与 CAS 数据不一致时，恢复流程会失败（可接受，需错误可诊断）。
- 新增 API 数量增加，需保持命名与调用意图清晰，避免与网络路径混淆。
