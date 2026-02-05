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
- [ ] 远端缓存策略（provider cache/republish）

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
- [ ] 观察者回放验证工具

### 8. 测试与验证
- [ ] 本地多节点集成测试（3 节点：exec+storage+gateway）
- [ ] 恢复测试（重启后继续同步 head）
- [ ] 数据一致性测试（state_root 校验）

## 依赖
- `doc/world-runtime.md`
- `doc/world-runtime/runtime-integration.md`
- `doc/world-runtime/module-storage.md`
- libp2p 协议栈与实现

## 状态
- 当前阶段：P1.7.2 完成（事件订阅与 head 追踪）
- 下一步：P1.7.3（观察者回放验证工具）
- 最近更新：事件订阅与 head 追踪实现（2026-02-05）
