# 基于 world_chain_runtime 的长跑脚本可用化（2026-02-28）

## 目标
- 将当前被阻断的长跑脚本恢复为“可执行、可产物、可门禁”的可用状态。
- 启动链路统一到 `world_chain_runtime`，不再依赖已下线的 `world_viewer_live --node-*` 参数族。
- 保留原脚本在自动化中的关键契约：可运行时长控制、日志目录、summary/timeline 产物、失败可定位。

## 范围
- 脚本改造：
  - `scripts/s10-five-node-game-soak.sh`
  - `scripts/p2p-longrun-soak.sh`
- 采样来源：从旧 epoch 报表文件改为 `world_chain_runtime` HTTP 接口：
  - `/v1/chain/status`
  - `/v1/chain/balances`
- 保留原有 chaos 注入能力（针对 p2p 脚本），并在新链路下复用。

## 非目标
- 本轮不恢复 `world_viewer_live` 内嵌节点路径。
- 本轮不追求与旧脚本所有字段完全 1:1 对齐，仅保证核心门禁与产物可用。

## 接口 / 数据
- 进程编排：多实例 `world_chain_runtime`。
- 每节点必需参数：
  - `--node-id --world-id --status-bind --node-role --node-tick-ms`
  - `--node-validator <id:stake>`（repeatable）
  - `--node-gossip-bind --node-gossip-peer`（多节点时）
- 采样字段（status）：
  - `running/tick_count/consensus.committed_height/network_committed_height/known_peer_heads/last_error`
- 采样字段（balances）：
  - `reward_mint_record_count/node_power_credit_balance/node_main_token_liquid_balance`
- 产物约定（保持兼容语义）：
  - `run_config.json`
  - `timeline.csv`
  - `summary.json`
  - `summary.md`
  - `failures.md`（失败时）

## 里程碑
- M1：建档（设计 + 项目管理）。
- M2：S10 脚本改造为可执行 `world_chain_runtime` 五节点版本。
- M3：P2P 脚本改造为可执行 `world_chain_runtime` 版本（含 chaos）。
- M4：回归与文档收口。

## 风险
- 新链路缺少旧 epoch 报表中的部分细分字段（例如 DistFS challenge 统计）。
  - 缓解：门禁先聚焦共识推进、连接健康、mint 记录与高度滞后；缺失字段标注 `unavailable`。
- 脚本参数较多，兼容策略可能引发旧调用预期偏差。
  - 缓解：help 明确“兼容字段/弃用字段”并给出默认行为。
- 多节点启动时序可能引入短期抖动。
  - 缓解：增加 startup grace + 重试采样窗口，避免误判。
