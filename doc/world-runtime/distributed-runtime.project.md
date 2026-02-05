# Agent World Runtime：分布式计算与存储（项目管理文档）

## 任务拆解
### 0. 对齐与准备
- [x] 输出设计文档（`doc/world-runtime/distributed-runtime.md`）
- [x] 输出项目管理文档（本文件）
- [x] 与 runtime 分册术语对齐（事件/快照/模块存储）

### 1. 协议与数据结构冻结
- [ ] 冻结 gossipsub topic 与 rr 协议命名
- [ ] 固化消息封装（ActionEnvelope/WorldHead/BlockAnnounce）
- [ ] 定义错误码与重试语义映射
- [ ] 选择并固化 wire encoding（CBOR/二进制）

### 2. 内容寻址存储与分片
- [ ] 抽象 BlobStore 接口（put/get/has）
- [ ] 快照/日志分片与 manifest 生成
- [ ] provider 发布与缓存策略（pin/evict）

### 3. 执行节点协同
- [ ] 执行节点拉取 wasm/state/journal 的 libp2p 适配器
- [ ] 执行结果写入 storage（block/journal/snapshot）
- [ ] 头指针更新与回放校验流程

### 4. Sequencer 与批处理
- [ ] action mempool 聚合与去重
- [ ] 批次生成与排序规则
- [ ] 租约式单写者切换与超时恢复

### 5. 索引与发现
- [ ] world head 发布到 DHT
- [ ] 内容 provider 索引与查询
- [ ] 轻量 index store 接口（可选）

### 6. Gateway/Observer
- [ ] 提交 action 的网关 API
- [ ] 事件订阅与 head 追踪
- [ ] 观察者回放验证工具

### 7. 测试与验证
- [ ] 本地多节点集成测试（3 节点：exec+storage+gateway）
- [ ] 恢复测试（重启后继续同步 head）
- [ ] 数据一致性测试（state_root 校验）

## 依赖
- `doc/world-runtime.md`
- `doc/world-runtime/runtime-integration.md`
- `doc/world-runtime/module-storage.md`
- libp2p 协议栈与实现

## 状态
- 当前阶段：D1（协议与消息草案完成）
- 下一步：P1（libp2p 协议/存储适配器原型）
- 最近更新：补充分布式协议命名/消息结构/错误码草案（2026-02-05）
