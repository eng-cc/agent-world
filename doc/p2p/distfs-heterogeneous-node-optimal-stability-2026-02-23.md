# Agent World Runtime：异构节点分布式存储最优稳定性改造（2026-02-23）

## 目标
- 面向“1000+ 节点、容量与在线时长显著异构”的场景，构建可长期稳定运行的分布式存储策略。
- 将当前“provider 列表 + 单节点优先请求”升级为“能力感知排序 + 多候选重试 + 退避回退”，降低单点离线与弱节点抖动影响。
- 在不破坏现有协议兼容的前提下，为后续“容量感知副本放置 / 自动修复”预埋数据结构。

## 范围

### In Scope
- `agent_world_proto`
  - 扩展 `ProviderRecord`，新增可选节点能力画像（容量、可用空间、在线率、挑战通过率、负载、延迟）。
- `agent_world_net`
  - 新增 provider 评分/排序策略模块。
  - `DistributedClient` 的 DHT 拉取路径升级为：按评分排序后逐节点定向重试（失败自动降级）。
  - 保留现有无画像节点的兼容行为（按时间新鲜度优先）。
- 测试
  - 增补排序与重试策略单测。
  - 回归 `agent_world_net`、`agent_world_distfs`、`agent_world_consensus`、`agent_world_node` 关键套件。

### Out of Scope
- 完整自动分片放置与在线重平衡调度（另行迭代）。
- 纠删码（Erasure Coding）与 PoSt 级证明协议完整化。
- 运维编排系统（跨机自动发现、弹性扩缩容控制面）。

## 接口/数据

### 1) Provider 能力画像（向后兼容）
- `ProviderRecord` 新增可选字段：
  - `storage_total_bytes: Option<u64>`
  - `storage_available_bytes: Option<u64>`
  - `uptime_ratio_per_mille: Option<u16>`
  - `challenge_pass_ratio_per_mille: Option<u16>`
  - `load_ratio_per_mille: Option<u16>`
  - `p50_read_latency_ms: Option<u32>`
- 兼容策略：
  - 历史数据缺失新字段时默认 `None`。
  - 排序策略对 `None` 使用中性分，确保旧节点可继续参与。

### 2) Provider 评分策略
- 新增 `ProviderSelectionPolicy`：
  - `freshness_ttl_ms`
  - `weight_freshness / weight_uptime / weight_challenge / weight_capacity / weight_load / weight_latency`
  - `max_candidates`
- 评分输出：
  - 归一化分值 `0.0~1.0`。
  - 可按分数降序给出候选 provider 序列。

### 3) 拉取重试策略
- `DistributedClient::fetch_blob_from_dht` 调整为：
  1. 读取 provider 列表。
  2. 基于策略排序。
  3. 逐 provider 定向请求（单 provider）并重试，直至成功。
  4. 全部失败后，回退到原始无 provider 拉取路径。
- 目标：降低“列表首节点离线/抖动”导致的失败概率。

## 里程碑
- M0：设计与任务拆解。
- M1：ProviderRecord 能力画像扩展与兼容。
- M2：评分排序与逐节点重试落地。
- M3：回归测试 + 文档/devlog 收口。

## 风险
- 评分策略权重不当可能引入倾斜（热点节点过载）。
  - 缓解：引入 `load_ratio_per_mille` 负反馈项，保留可配置权重。
- 能力画像缺失导致排序不稳定。
  - 缓解：`None` 字段按中性分处理，不阻断请求。
- 逐节点重试增加请求时延上界。
  - 缓解：`max_candidates` 限制重试范围，最终保留无 provider 回退路径。

## 当前状态
- 状态：进行中
- 已完成：M0、M1
- 进行中：M2
- 未开始：M3
