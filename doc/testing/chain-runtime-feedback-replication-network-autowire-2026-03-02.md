# Chain Runtime 反馈网络复制层自动挂载修复（2026-03-02）

## 目标
- 修复 `world_chain_runtime` 启动失败：默认启用 `feedback_p2p` 后，因未挂载 replication network 导致 `InvalidConfig("feedback_p2p requires replication network")`。
- 保证单机链运行时在无外部 peers 时也可完成 feedback announce/fetch 闭环，不阻断启动器链路。

## 范围
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/bin/world_chain_runtime/world_chain_runtime_tests.rs`
- `doc/testing/chain-runtime-feedback-replication-network-autowire-2026-03-02.project.md`
- `doc/devlog/2026-03-02.md`

## 接口 / 数据
- 新增默认 replication network 挂载逻辑（`world_chain_runtime` 内部）：
  - 默认 listen 地址：`/ip4/127.0.0.1/tcp/0`
  - 使用 `Libp2pReplicationNetworkConfig`
  - 启用 `allow_local_handler_fallback_when_no_peers=true`
- 启动时序调整：
  - `NodeRuntime::new(config)` 后先挂载 `NodeReplicationNetworkHandle`
  - 再执行 `runtime.start()`
- 不修改现有 CLI 参数与 HTTP 接口。

## 里程碑
- M1：建档（设计文档 + 项目管理文档）。
- M2：完成 `world_chain_runtime` replication network 自动挂载修复。
- M3：补充/更新测试并完成启动烟测。
- M4：文档状态与 devlog 收口。

## 风险
- 风险：默认 loopback + ephemeral 端口仅保证单机闭环，跨节点联通仍依赖后续拓扑参数化。
  - 缓解：保持实现为最小修复，不改变现有网络参数口径；后续可单独扩展 CLI。
- 风险：启用本地 handler fallback 可能掩盖“无 peers”配置问题。
  - 缓解：仅用于无 peer 时本地闭环；真实多节点联调仍可通过连通性指标与日志识别异常。

## 完成态（2026-03-02）
- `world_chain_runtime` 已在启动前自动挂载默认 replication network，`feedback_p2p` 不再因缺少 network handle 启动失败。
- 新增测试 `default_replication_network_config_uses_loopback_ephemeral_listen`，覆盖默认网络配置。
- 定向验证通过：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --bin world_chain_runtime -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world --bin world_chain_runtime`
  - `timeout 12s env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_chain_runtime -- --status-bind 127.0.0.1:6121`（观察到 `world_chain_runtime ready.`）
