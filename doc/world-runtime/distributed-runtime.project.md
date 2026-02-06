# Agent World Runtime：分布式计算与存储（项目管理文档）

## 任务拆解
### 0. 对齐与准备
- [x] 输出设计文档（`doc/world-runtime/distributed-runtime.md`）
- [x] 输出项目管理文档（本文件）
- [x] 与 runtime 分册术语对齐（事件/快照/模块存储）

### 1. 协议与数据结构冻结
- [x] 冻结 gossipsub topic 与 rr 协议命名
- [x] 固化消息封装（ActionEnvelope/WorldHead/BlockAnnounce）
- [x] 定义错误码与重试语义映射
- [x] 选择并固化 wire encoding（CBOR/二进制）

### 2. 内容寻址存储与分片
- [x] 抽象 BlobStore 接口（put/get/has）
- [x] 本地 CAS 参考实现（LocalCasStore）
- [x] 快照/日志分片与 manifest 生成
- [x] 本地 pin/evict 策略（pins.json + prune_unpinned）
- [x] provider 发布与索引（DHT provider）
- [x] 远端缓存策略（provider cache/republish）
  - [x] provider cache 与 TTL 规则
  - [x] provider republish 机制
  - [x] provider cache 测试

### 3. 执行节点协同
- [x] 执行节点拉取 wasm/state/journal 的网络客户端（基于 DistributedNetwork）
- [x] 执行结果写入 storage（block/journal/snapshot）
- [x] 头指针更新与回放校验流程

### 4. 网络适配器原型（P1）
- [x] 定义 DistributedNetwork 抽象接口
- [x] 提供 InMemoryNetwork 参考实现（测试用）
- [x] Libp2pNetwork 骨架（peer_id/keypair，占位实现）
- [x] DHT 适配器抽象 + InMemoryDht 参考实现
- [x] libp2p Swarm 事件循环（gossipsub + rr 基线）
- [x] libp2p 实现（gossipsub + rr + dht）

### 5. Sequencer 与批处理
- [x] action mempool 聚合与去重
- [x] 批次生成与排序规则
- [x] 租约式单写者切换与超时恢复

### 6. 索引与发现
- [x] world head 发布到 DHT（libp2p/kad 适配）
- [x] 内容 provider 索引与查询（libp2p/kad 适配）
- [x] 轻量 index store 接口（可选）

### 7. Gateway/Observer
- [x] 提交 action 的网关 API
- [x] 事件订阅与 head 追踪
- [x] 观察者回放验证工具

### 8. 测试与验证
- [x] 本地多节点集成测试（3 节点：exec+storage+gateway）
- [x] 恢复测试（重启后继续同步 head）
- [x] 数据一致性测试（state_root 校验）

### 9. P2.1 索引缓存封装
- [x] CachedDht 封装（provider/head 缓存）
- [x] TTL 与 provider 截断策略
- [x] CachedDht 单元测试

### 10. P2.2 观察者 DHT 引导
- [x] 观察者回放支持 DHT head 引导
- [x] DHT 引导单元测试

### 11. P2.3 ProviderCache 执行接入
- [x] 执行结果发布接入 ProviderCache
- [x] ProviderCache 发布单元测试

### 12. P2.4 Provider-aware 拉取
- [x] DistributedNetwork 支持 provider 列表请求
- [x] Observer 回放拉取使用 provider 列表
- [x] provider-aware 客户端单元测试

### 13. P2.5 Provider-aware 拉取封装
- [x] DistributedClient 提供 DHT 拉取封装
- [x] Observer 回放复用 DHT 拉取封装
- [x] DHT 拉取封装单元测试

### 14. P2.6 模块拉取封装
- [x] 模块 manifest 拉取走 DHT provider-aware 路径
- [x] 模块 artifact 拉取走 DHT provider-aware 路径
- [x] 模块拉取封装单元测试

### 15. P2.7 libp2p provider 重发
- [x] libp2p provider 重发配置与定时触发
- [x] provider 重发间隔单元测试

### 16. P2.8 模块加载补全
- [x] World 模块加载支持 DHT 拉取补全
- [x] 模块加载补全单元测试

### 17. P2.9 模块预热加载
- [x] World 预热活跃模块（DHT 拉取补全）
- [x] 模块预热加载单元测试

### 18. P3.1 治理模块补全
- [x] shadow/apply 支持 DHT 拉取缺失工件
- [x] 治理拉取补全单元测试

### 19. P3.2 执行节点启动引导
- [x] bootstrap_world_from_dht 启动引导
- [x] 启动引导单元测试

### 20. P3.3 Head 跟随与同步
- [x] head 选择策略（height/timestamp/block_hash）
- [x] HeadFollower 同步 helper（忽略重复/陈旧）
- [x] HeadFollower 单元测试

### 21. P3.4 Observer head 同步
- [x] ObserverClient 同步 helper（sync_heads/sync_heads_with_dht）
- [x] Observer head 同步单元测试

### 22. P3.5 Observer 同步结果回传
- [x] ObserverClient 同步结果 helper（sync_heads_with_result/sync_heads_with_dht_result）
- [x] Observer 同步结果单元测试

### 23. P3.6 Observer 同步报告
- [x] ObserverClient 同步报告 helper（sync_heads_report/sync_heads_with_dht_report）
- [x] Observer 同步报告单元测试

## 依赖
- `doc/world-runtime.md`
- `doc/world-runtime/runtime-integration.md`
- `doc/world-runtime/module-storage.md`
- libp2p 协议栈与实现

## 状态
- 当前阶段：P3.6 完成（Observer 同步报告）
- 下一步：P3.7（后续协议/实现迭代）
- 最近更新：Observer 同步报告（2026-02-05）
