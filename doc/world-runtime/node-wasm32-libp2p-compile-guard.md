# Agent World Runtime：Node libp2p wasm32 编译兼容守卫（设计文档）

## 目标
- 修复 `agent_world_node` 在 `wasm32-unknown-unknown` 目标上的编译失败，避免阻塞 pre-commit 的 Web Viewer wasm 编译门禁。
- 保持 native 目标下现有 libp2p replication 能力不变。
- 稳定 `viewer::web_bridge` 相关回归测试，消除提交门禁中的偶发 `WouldBlock/Disconnected` 失败。

## 范围
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/libp2p_replication_network_wasm.rs`（新增）
- `crates/agent_world/src/viewer/web_bridge.rs`
- `doc/world-runtime/node-wasm32-libp2p-compile-guard.project.md`
- `doc/devlog/2026-02-16.md`

不在范围：
- 调整 PoS/gossip 共识业务语义。
- 为 wasm 端实现真实 libp2p 传输能力。

## 接口/数据
- 对外接口保持不变：继续导出 `Libp2pReplicationNetwork` 与 `Libp2pReplicationNetworkConfig`。
- 在 `wasm32` 目标下提供等价 API 的最小占位实现：
  - `new/peer_id` 可用；
  - `DistributedNetwork` 能力返回 `NetworkProtocolUnavailable`（显式声明当前目标不支持 native libp2p replication）。

## 里程碑
- M1：完成文档立项与任务拆解。
- M2：完成 `agent_world_node` 的 wasm32 编译守卫与占位实现。
- M3：稳定 `web_bridge` 重连测试并完成回归验证（`agent_world_node` native check + `agent_world_viewer` wasm32 check）后收口文档。

## 风险
- 若后续在 wasm 侧误用该网络实现，运行期会收到 unavailable 错误；需由调用方按目标区分能力。
- 占位实现需保持接口稳定，避免对现有 native 调用路径造成回归。
- 测试稳定性修复依赖本地调度时序，需避免再次引入短超时或非阻塞 socket 继承导致的间歇失败。
