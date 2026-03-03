# Agent World Runtime：分布式存储去单机完整依赖改造（2026-02-23）

## 目标
- 明确移除“任意单机可独立提供完整执行数据”的隐含假设。
- DHT 路径下的数据拉取必须依赖 provider 索引，不允许无 provider 时回退到非定向全网请求。
- 在回放/启动校验链路中增加分布覆盖约束，避免“单节点全覆盖”成为系统可接受常态。

## 范围

### In Scope
- `agent_world_net::DistributedClient`
  - `fetch_blob_from_dht` 改为严格 DHT provider 模式：
    - 无 provider -> 失败；
    - provider 全失败 -> 失败；
    - 不再回退 `fetch_blob(content_hash)`。
- `agent_world_net`
  - 新增执行数据 provider 覆盖审计模块：
    - 每个 blob 最小副本 provider 数（`min_replicas_per_blob`）。
    - 禁止单 provider 覆盖整套执行数据（`forbid_single_provider_full_coverage`）。
  - 在 DHT 批量拉取入口 `fetch_blobs_from_dht_with_distribution` 接入覆盖审计。
- 测试
  - 严格 DHT 拉取失败/成功路径。
  - 覆盖审计拒绝“单节点全覆盖”与“副本不足”。
  - 覆盖审计放行满足分布约束的数据集。

### Out of Scope
- 自动副本修复调度。
- 分片迁移与负载重平衡控制面。
- 纠删码编码/解码协议改造。

## 接口/数据

### 1) 严格 DHT 拉取语义
- `DistributedClient::fetch_blob_from_dht`：
  - 输入：`world_id/content_hash/dht`。
  - 行为：仅按 DHT provider 列表排序后逐节点重试。
  - 失败语义：
    - provider 为空：`DistributedValidationFailed`；
    - 全 provider 失败：返回最后一次错误或统一 `DistributedValidationFailed`。

### 2) 分布覆盖策略
- 新增 `ProviderDistributionPolicy`：
  - `min_replicas_per_blob: usize`（默认 2）
  - `forbid_single_provider_full_coverage: bool`（默认 true）
- 审计输入：`world_id` + 一组执行数据 hash（block/snapshot_manifest/journal_segments/chunks/segments）。
- 审计输出：
  - 通过：覆盖满足约束。
  - 失败：给出副本不足 hash 与违规 provider 信息。

## 里程碑
- M0：设计与任务拆解（已完成）。
- M1：严格 DHT 拉取落地（去掉单机回退，已完成）。
- M2：覆盖审计模块与 DHT 批量拉取接线（已完成）。
- M3：回归、文档与日志收口（已完成）。

## 风险
- 风险：严格模式会暴露历史环境中 provider 注册不全问题。
  - 缓解：错误信息显式包含缺失 hash，便于运维补齐 provider 发布流程。
- 风险：覆盖审计增加 DHT 查询开销。
  - 缓解：仅在启用批量 DHT 拉取分布审计时触发，默认单 blob 读取不引入额外查询。

## 当前状态
- 状态：已完成
- 已完成：M0、M1、M2、M3
- 进行中：无
- 未开始：无
