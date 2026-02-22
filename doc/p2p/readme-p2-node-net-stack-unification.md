# README P2 缺口收口：Node Replication 统一到 agent_world_net 网络栈（设计文档）

## 目标
- 收口此前评估中的“网络栈分裂”问题：将 `agent_world_node` 的 replication libp2p 实现从独立 swarm 线程迁移为复用 `agent_world_net::Libp2pNetwork`。
- 保持 `world_viewer_live` 现有接线和配置不变，不改变上层对 `Libp2pReplicationNetwork` 的调用方式。
- 保持 node replication 现有关键语义不回退：
  - 多 peer 轮换请求；
  - 远端失败后自动重试下一个 peer；
  - `allow_local_handler_fallback_when_no_peers` 可控本地回退策略。
- 明确 wasm32 目标定位：`agent_world_node` 在 wasm32 下不做 full node 网络协议栈，仅提供编译占位与 unavailable 返回。

## 范围
- In scope
  - `crates/agent_world_node/src/libp2p_replication_network.rs`
    - 改为对 `agent_world_net::Libp2pNetwork` 的封装。
    - 在封装层补齐 node 语义（轮换/重试/回退）。
  - `crates/agent_world_node/Cargo.toml`
    - 增加 `agent_world_net`（仅 native 目标依赖）。
  - `crates/agent_world_node/src/libp2p_replication_network_wasm.rs`
    - 增加注释，明确 wasm32 非 full node。
  - 文档
    - `doc/p2p/readme-p2-node-net-stack-unification.project.md`
    - `doc/p2p/node-wasm32-libp2p-compile-guard.md`（补充 wasm32 定位）
- Out of scope
  - wasm32 端完整 libp2p full node（包含 TCP/Noise/Yamux 与 DHT/自动发现等）。
  - `agent_world_net` 的 DHT/索引语义重构。
  - NAT 穿透、生产运维编排、PKI/KMS。

## 接口 / 数据
### 1) `Libp2pReplicationNetwork` 对外接口保持不变
- 保持已有类型名与 `DistributedNetwork<WorldError>` 实现不变：
  - `Libp2pReplicationNetworkConfig`
  - `Libp2pReplicationNetwork`
- 配置字段保持不变：
  - `keypair`
  - `listen_addrs`
  - `bootstrap_peers`
  - `allow_local_handler_fallback_when_no_peers`

### 2) 请求路由语义（封装层负责）
- 连接 peer 集合非空时：
  - 按轮换顺序选择 peer；
  - 单 peer 请求失败或返回错误响应时，自动重试下一个 peer。
- 连接 peer 集合为空时：
  - 默认返回 `NetworkProtocolUnavailable`（包含 no connected peers 语义）；
  - 仅在 `allow_local_handler_fallback_when_no_peers=true` 时允许本地 handler 回退。

### 3) 远端错误识别
- 底层复用 `agent_world_net::Libp2pNetwork` request/response。
- 封装层在收到响应 payload 后尝试按 `ErrorResponse` 解码：
  - 命中则视为远端处理失败，触发重试或返回 `NetworkRequestFailed`；
  - 未命中则视为业务 payload 成功。

## 里程碑
- M1：完成设计文档 + 项目管理文档，冻结迁移边界。
- M2：完成 node replication 底层迁移与封装层语义补齐。
- M3：完成 `agent_world_node` / `agent_world_net` 回归与文档/devlog 收口。

## 风险
- 兼容风险：迁移后底层 transport 语义由 `agent_world_net` 承担，需通过回归测试确保 node 的重试/回退行为不回退。
- 误判风险：远端错误以 `ErrorResponse` 识别，需避免将正常 payload 误判为错误响应。
- 构建风险：`agent_world_node` 新增 `agent_world_net` 依赖时，需限制在 native 目标，避免影响 wasm32 编译路径。
