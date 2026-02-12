# Agent World Runtime：`agent_world_net` + `agent_world_consensus` 拆分（设计文档）

## 目标
- 将分布式能力按职责拆为两个 crate：
  - `agent_world_net`：网络与传输相关能力。
  - `agent_world_consensus`：共识与成员同步相关能力。
- 明确将 `distributed_membership_sync` 归类到 `agent_world_consensus`。
- 保持 `agent_world` 对外 API 兼容，避免一次性大范围破坏。

## 范围

### In Scope（本次）
- 新建 crate：
  - `crates/agent_world_net`
  - `crates/agent_world_consensus`
- 将两个 crate 加入 workspace，并形成稳定导出入口（按能力分类导出）。
- `agent_world_consensus` 对外聚合共识与 membership sync 能力（含 `distributed_membership_sync` 相关导出）。
- 补充最小验证（编译/测试）与文档回写。

### In Scope（扩展阶段）
- 在不引入第三个 crate 的前提下，继续推进“实现物理迁移”：
  - 优先将 `distributed_net` 的核心实现（`DistributedNetwork` / `InMemoryNetwork`）下沉到 `agent_world_net`。
  - `agent_world` 保持兼容 API，不改变现有对外行为与语义。
- 通过定向编译/测试验证迁移不回归。

### In Scope（二次扩展阶段）
- 继续推进 `agent_world_net` 的网络基础设施下沉：
  - 将 `distributed_dht` 的核心实现（`DistributedDht` / `InMemoryDht`）下沉到 `agent_world_net`。
  - `MembershipDirectorySnapshot` / `ProviderRecord` 由 `agent_world_proto` 驱动，避免重复协议定义。
- 保持 `agent_world` 与现有测试行为兼容；迁移过程允许短期共存（runtime 侧保留实现，新 crate 侧提供并行实现）。

### In Scope（三次扩展阶段）
- 继续推进 `agent_world_net` 的网络访问层下沉：
  - 将 `distributed_client` 核心实现（`DistributedClient` 及其 DHT provider 路由逻辑）下沉到 `agent_world_net`。
  - 在 `agent_world_net` 内补齐 canonical CBOR 序列化 helper，避免依赖 `agent_world::runtime::util` 私有实现。
- 保持 `agent_world` 与现有测试行为兼容；迁移过程允许短期共存（runtime 侧保留实现，新 crate 侧提供并行实现）。

### In Scope（四次扩展阶段）
- 继续推进 `agent_world_net` 的网关能力下沉：
  - 将 `distributed_gateway` 核心实现（`ActionGateway` / `NetworkGateway` / `SubmitActionReceipt`）下沉到 `agent_world_net`。
  - 复用 `agent_world_proto::distributed` 的 topic helper 与 `ActionEnvelope` 协议类型，保持 wire 行为一致。
- 保持 `agent_world` 与现有测试行为兼容；迁移过程允许短期共存（runtime 侧保留实现，新 crate 侧提供并行实现）。

### In Scope（五次扩展阶段）
- 继续推进 `agent_world_net` 的索引发布能力下沉：
  - 将 `distributed_index` 核心实现（`IndexPublishResult`、head/provider 发布与查询 helper）下沉到 `agent_world_net`。
  - 保持 `ExecutionWriteResult` 输入契约兼容，确保现有执行结果索引路径不回归。
- 保持 `agent_world` 与现有测试行为兼容；迁移过程允许短期共存（runtime 侧保留实现，新 crate 侧提供并行实现）。

### In Scope（六次扩展阶段）
- 在不新增 crate 的前提下，先完成 `agent_world_net` 内部模块化拆分：
  - 将 `src/lib.rs` 按能力拆成 `network` / `dht` / `index` / `client` / `gateway` / `util` 多文件结构。
  - 将 `agent_world_net` 单测拆分到独立 `tests.rs`，保证 `lib.rs` 行数安全余量，避免后续迁移触及 1200 行约束。
- 保持 `agent_world_net` 对外导出 API 不变，确保 `agent_world` 调用方无需改动。

### In Scope（七次扩展阶段）
- 继续推进 `agent_world_net` 的缓存与索引存储能力下沉：
  - 将 `distributed_index_store` 核心实现（`DistributedIndexStore` / `HeadIndexRecord` / `InMemoryIndexStore`）下沉到 `agent_world_net`。
  - 将 `distributed_provider_cache` 核心实现（`ProviderCache` / `ProviderCacheConfig`）下沉到 `agent_world_net`。
  - 将 `distributed_dht_cache` 核心实现（`CachedDht` / `DhtCacheConfig`）下沉到 `agent_world_net`。
- 保持 `agent_world_net` 对外导出名与调用语义不变，确保 `agent_world` 调用方无需改动。

### Out of Scope（本次不做）
- 不在本轮强制把 `agent_world` 现有 runtime 实现文件全部物理迁移到新 crate。
- 不做协议层额外重构（协议仍以 `agent_world_proto` 为主）。
- 不改动业务语义与现有行为策略。

## 接口 / 数据

### `agent_world_net`（边界）
- 负责导出：
  - 网络抽象与 in-memory 网络实现。
  - 分布式网络客户端/网关/观察者等网络链路能力。
  - 可选 `libp2p` 能力导出（feature 透传）。

### `agent_world_consensus`（边界）
- 负责导出：
  - 共识主流程（提案/投票/提交）相关能力。
  - 成员目录同步与恢复能力。
  - 吊销对账、调度、告警、恢复、审计相关能力。
- `distributed_membership_sync` 及其子模块归属此 crate 的能力面。

## 里程碑
- P1：设计文档与项目管理文档完成。
- P2：新 crate 脚手架与 workspace 接线完成。
- P3：导出能力面落地（net/consensus 分类稳定）。
- P4：编译与定向测试回归通过，项目文档收口。
- P5：`distributed_net` 核心实现下沉到 `agent_world_net`。
- P6：扩展阶段回归验证与文档收口。
- P7：`distributed_dht` 核心实现下沉到 `agent_world_net`。
- P8：二次扩展阶段回归验证与文档收口。
- P9：`distributed_client` 核心实现下沉到 `agent_world_net`。
- P10：三次扩展阶段回归验证与文档收口。
- P11：`distributed_gateway` 核心实现下沉到 `agent_world_net`。
- P12：四次扩展阶段回归验证与文档收口。
- P13：`distributed_index` 核心实现下沉到 `agent_world_net`。
- P14：五次扩展阶段回归验证与文档收口。
- P15：完成 `agent_world_net` 内部模块化拆分（多文件 + 单测迁移）。
- P16：六次扩展阶段回归验证与文档收口。
- P17：`distributed_index_store` / `distributed_provider_cache` / `distributed_dht_cache` 核心实现下沉到 `agent_world_net`。
- P18：七次扩展阶段回归验证与文档收口。

## 风险
- 仅做边界导出时，可能出现“新 crate 已存在但实现仍在 `agent_world`”的过渡期认知偏差。
- 导出项过多时，维护成本上升；需要后续按使用频率做分组收敛。
- feature 透传（尤其 `libp2p`）若配置不完整，可能导致构建行为与预期不一致。
- 迁移期间若同名类型在 runtime 与新 crate 并行存在，可能引入调用方误用；需要通过文档和导出边界降低歧义。
- `distributed_client` 涉及协议编解码与错误映射，若 canonical CBOR 行为偏差可能导致跨节点请求兼容性回归，需要定向测试覆盖。
- `distributed_gateway` 涉及 action 发布 topic 与序列化，若 topic 生成或 payload 编码偏差，可能导致上游节点收不到动作，需要保留端到端发布测试。
- `distributed_index` 涉及执行产物 hash 聚合与 provider 发布，若 hash 收集集合不一致会导致拉取路径缺 provider，需要保留执行产物全量索引测试。
- `agent_world_net` 模块化拆分若处理不当可能引入循环依赖或可见性收窄，需保持导出面稳定并保留全量 `agent_world_net` 单测回归。
- cache/index store 下沉涉及 TTL、provider 截断与 head 缓存刷新路径，若时间窗口判定偏差会导致命中过期数据或频繁回源，需保留缓存命中/过期刷新测试。
