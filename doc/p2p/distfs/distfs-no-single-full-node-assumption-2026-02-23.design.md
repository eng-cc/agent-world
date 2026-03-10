# DistFS 去单机完整依赖设计

- 对应需求文档: `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.project.md`

## 1. 设计定位
定义 DistFS 从“存在单个完整节点兜底”转向“严格依赖 DHT provider 分布覆盖”的方案，确保 blob 拉取和覆盖审计不再静默回退单机。

## 2. 设计结构
- 严格拉取层：`fetch_blob_from_dht` 仅按 provider 列表重试，不再回退 `fetch_blob(content_hash)`。
- 覆盖策略层：`ProviderDistributionPolicy` 约束最小副本数和禁止单 provider 全覆盖。
- 批量审计层：DHT 批量拉取入口附带覆盖审计，输出缺失 hash 与违规 provider。
- 运维暴露层：严格模式把历史 provider 注册不全问题显式暴露。

## 3. 关键接口 / 入口
- `DistributedClient::fetch_blob_from_dht`
- `ProviderDistributionPolicy`
- `fetch_blobs_from_dht_with_distribution`
- 覆盖审计输出

## 4. 约束与边界
- 不引入自动副本修复、分片迁移和纠删码改造。
- 严格模式会带来更早失败，这是有意的治理强化。
- 单 blob 读取不应引入额外无谓查询开销。
- 该设计与异构节点稳定性专题互补，不重复定义节点分层。

## 5. 设计演进计划
- 先去掉单机回退。
- 再接分布覆盖审计。
- 最后通过回归和运维错误口径收口严格 DHT 模式。
