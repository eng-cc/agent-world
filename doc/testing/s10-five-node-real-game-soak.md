# Agent World：S10 五节点真实游戏数据在线长跑套件（设计文档）

## 目标
- 在现有 S9（P2P/存储/共识长跑）之上，新增面向“真实游戏数据流”的 S10 套件。
- 用 5 节点（1 sequencer + 2 storage + 2 observer）在线模拟，验证真实运行期的数据交换、共识推进、结算分发与不变量。
- 提供可重复执行的一键脚本和统一证据产物，作为发布前高风险门禁补充。

## 范围

### In Scope
- 新增 S10 编排脚本：`scripts/s10-five-node-game-soak.sh`。
- 五节点拓扑采用 `world_viewer_live --topology single` 多进程编排：
  - `s10-sequencer`
  - `s10-storage-a`
  - `s10-storage-b`
  - `s10-observer-a`
  - `s10-observer-b`
- 默认场景使用 `llm_bootstrap`（可切换），保障运行期存在真实 gameplay 资源状态变化。
- 输出统一产物目录：运行配置、时间线、汇总结果、失败清单、每节点日志与 epoch 报表。
- 将 S10 接入 `testing-manual.md`，明确与 S9 的边界与执行口径。

### Out of Scope
- 不改造 `world_viewer_live` 拓扑枚举（不新增 `penta_distributed` 内建模式）。
- 不在本轮引入 S10 chaos 注入编排（先完成稳定基线）。
- 不引入新的共识算法或新的存储证明协议。

## 接口 / 数据

### 1) S10 脚本入口
- `./scripts/s10-five-node-game-soak.sh [options]`
- 关键参数：
  - `--duration-secs <n>`：运行时长（默认 1800s）。
  - `--scenario <name>`：默认 `llm_bootstrap`。
  - `--llm` / `--no-llm`：是否启用 LLM 决策（默认 `--no-llm`，走 script fallback）。
  - `--base-port <n>`：端口基准（默认 5810）。
  - `--out-dir <path>`：输出目录（默认 `.tmp/s10_game_longrun`）。
  - `--max-stall-secs <n>`、`--max-lag-p95 <n>`、`--max-distfs-failure-ratio <0~1>`：门禁阈值。
  - `--dry-run`：仅输出配置与命令，不启动进程。

### 2) 拓扑与验证人配置
- 固定 5 节点验证人集合（脚本内生成并注入每个节点）：
  - `s10-sequencer:35`
  - `s10-storage-a:20`
  - `s10-storage-b:20`
  - `s10-observer-a:15`
  - `s10-observer-b:10`
- 每节点配置 gossip bind + full-mesh peers，形成真实网络消息交换路径。

### 3) S10 门禁指标
- 共识：`committed_height` 单调推进，`stall <= max_stall_secs`。
- 网络追平：`lag_p95(network_committed_height - committed_height) <= max_lag_p95`。
- 存储挑战：`distfs_failure_ratio <= max_distfs_failure_ratio`（在有样本时）。
- 资产一致性：`reward_asset_invariant_status.ok == true`。
- 真实数据交换：
  - 至少出现一次 `settlement_report.total_distributed_points > 0`；
  - 至少出现一次 `minted_records` 非空。

### 4) 产物目录
- `<out-dir>/<timestamp>/`
  - `run_config.json`
  - `timeline.csv`
  - `summary.json`
  - `summary.md`
  - `failures.md`（失败时）
  - `nodes/<node_id>/{command.txt,stdout.log,stderr.log,report/*.json}`

## 里程碑
- M0：设计文档与项目管理文档建档。
- M1：S10 五节点编排脚本落地（启动/停止/汇总）。
- M2：S10 指标门禁与证据产物落地。
- M3：`testing-manual.md` 接线与执行口径收口。

## 风险
- 单机五进程资源竞争可能导致短时抖动。
  - 缓解：阈值分层、记录完整 timeline 与节点 stderr。
- LLM 模式依赖外部 API，可能引入非确定性。
  - 缓解：默认 `--no-llm`，需要时显式 `--llm`。
- 无 chaos 注入时对恢复能力覆盖不足。
  - 缓解：S10 先做稳定基线，恢复能力继续由 S9 chaos 套件承担。
