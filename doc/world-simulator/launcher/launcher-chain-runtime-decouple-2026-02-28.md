# 启动器链路重构：链运行时与游戏进程解耦（2026-02-28）

## 目标
- 将 P2P 节点/链运行时从游戏进程层（`world_viewer_live`）中拆出，改为独立进程承载。
- `world_game_launcher` 默认一键拉起“游戏服务 + 链运行时 + Web 静态服务”，实现对外统一发行入口。
- 在 launcher 链路中提供链配置能力，并暴露可观测接口（含 token 余额视图）。

## 范围
- 新增独立二进制：`world_chain_runtime`（归属 `agent_world` crate）。
- `world_chain_runtime` 负责：
  - 启动/停止 NodeRuntime；
  - 维护执行世界（execution world）落盘路径；
  - 暴露 HTTP 状态接口（health/status/balances）。
- `world_game_launcher` 负责：
  - 启动并托管 `world_chain_runtime` 子进程（默认启用）；
  - 启动 `world_viewer_live` 时改为 `--no-node`；
  - 透传链配置参数并输出链状态入口 URL。
- 更新发行打包脚本：将 `world_chain_runtime` 纳入 bundle。

## 非目标
- 本轮不实现复杂钱包 UI、助记词管理、链上转账签名流程。
- 本轮不替换现有 `world_viewer_live` 内全部节点代码路径（保留兼容 CLI 场景）。
- 本轮不覆盖跨机器大规模集群编排（先覆盖单机发行链路）。

## 接口 / 数据
### 1) `world_chain_runtime` CLI（新增）
- `--node-id <id>`：默认 `viewer-live-node`。
- `--world-id <id>`：默认 `live-llm_bootstrap`。
- `--status-bind <host:port>`：默认 `127.0.0.1:5121`。
- `--node-role <sequencer|storage|observer>`：默认 `sequencer`。
- `--node-tick-ms <n>`：默认 `200`。
- `--node-validator <id:stake>`：可重复；为空时默认单 validator（本节点）。
- `--node-auto-attest-all` / `--node-no-auto-attest-all`。
- `--node-gossip-bind <addr:port>` / `--node-gossip-peer <addr:port>`（可选）。
- `--execution-world-dir <path>`：默认 `output/chain-runtime/execution-world`。
- `--no-openapi`：可选（当前仅纯 JSON HTTP）。

### 2) `world_chain_runtime` HTTP 接口（新增）
- `GET /healthz`：存活探针。
- `GET /v1/chain/status`：节点运行状态、共识高度、错误信息。
- `GET /v1/chain/balances`：从 execution world 读取
  - `node_asset_balances`
  - `reward_mint_records`（最近记录）
  - `main_token_balances`（若存在）

### 3) `world_game_launcher` CLI 扩展
- `--chain-enable` / `--chain-disable`（默认 enable）。
- `--chain-status-bind <host:port>`：默认 `127.0.0.1:5121`。
- `--chain-node-id <id>`。
- `--chain-world-id <id>`（默认跟随 scenario 推导）。
- `--chain-node-role <role>`。
- `--chain-node-tick-ms <n>`。
- `--chain-node-validator <id:stake>`（repeatable）。

### 4) 关键链路
- 桌面入口：`run-client.sh -> agent_world_client_launcher -> world_game_launcher`。
- CLI 入口：`run-game.sh -> world_game_launcher`。
- 启动器编排：`world_game_launcher -> world_chain_runtime + world_viewer_live(--no-node) + static_http`。

## 里程碑
- M1：`world_chain_runtime` 落地（节点主循环 + status/balances API）。
- M2：`world_game_launcher` 完成链子进程托管，viewer 进程切换为 `--no-node`。
- M3：发行打包与桌面启动器参数透传完成。
- M4：回归测试、文档收口、发布口径验证。

## 风险
- 运行时状态拆分后，链状态与游戏状态可能出现可观测时序差。
  - 缓解：status/balances 接口明确 `observed_at_unix_ms` 与数据来源路径。
- 默认启用链运行时会增加启动依赖（端口冲突、文件权限）。
  - 缓解：提供 `--chain-disable` 快速降级路径。
- token 余额来源于 execution world，早期 run 可能为空。
  - 缓解：接口显式返回空集合与原因字段，不视为错误。
