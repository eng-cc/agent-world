# 发布门禁指标策略对齐（2026-02-28）

## 目标
- 修复 S9/S10 在 `world_chain_runtime` 链路下“有运行闭环但指标缺失”的门禁误判。
- 让门禁直接消费真实 reward runtime 指标（mint、distfs、settlement、invariant），而不是仅依赖兼容字段或放宽策略。

## 范围
- 运行时改造：
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/reward_runtime_worker.rs`
- 脚本改造：
  - `scripts/p2p-longrun-soak.sh`
  - `scripts/s10-five-node-game-soak.sh`
- 运行手册与项目文档同步：
  - `testing-manual.md`
  - `doc/devlog/2026-02-28.md`

## 非目标
- 不改动 `third_party/`。
- 不重构奖励网络为跨节点签名服务（当前只需本地 runtime worker 产出门禁可用指标）。

## 接口 / 数据
- `world_chain_runtime` 新增参数：
  - `--reward-runtime-enable|--reward-runtime-disable`
  - `--reward-runtime-signer-node-id`
  - `--reward-runtime-epoch-duration-secs`
  - `--reward-points-per-credit`
  - `--reward-runtime-auto-redeem|--reward-runtime-no-auto-redeem`
  - `--reward-initial-reserve-power-units`
  - `--reward-distfs-*`（probe/adaptive 族参数）
- `GET /v1/chain/status` 新增 `reward_runtime` 快照，包含：
  - `metrics_available`、`cumulative_minted_record_count`
  - `distfs_total_checks`、`distfs_failed_checks`
  - `settlement_apply_attempts_total`、`settlement_apply_failures_total`
  - `invariant_ok`、`last_error`
- S9/S10 脚本门禁改造：
  - 启动命令透传 `--reward-runtime-epoch-duration-secs`、`--reward-points-per-credit`。
  - mint 样本使用 `max(balances.reward_mint_record_count, reward_runtime.cumulative_minted_record_count)`。
  - distfs 与 settlement 比例使用“每节点累计最大值后聚合”计算。
  - 产物结构保持兼容：`run_config.json`、`summary.json`、`summary.md`、`failures.md` 路径不变。

## 里程碑
- M1：建档（设计 + 项目管理）。
- M2：接通 runtime reward worker 与 status 指标输出。
- M3：脚本切换到真实 reward runtime 指标并修正 chaos 误判。
- M4：复跑 S9/S10 发布门禁命令并确认通过。
- M5：手册与项目文档收口，补任务日志。

## 风险
- `distfs_total_checks` 在短窗口可能为 0，造成 `insufficient_data`/告警噪声。
  - 缓解：保留门禁阈值但将“数据未就绪”与“比例超阈值”分离，避免误杀。
- chaos 注入可能触发 `running_false`/`http_failure`，误伤 reward invariant 判定。
  - 缓解：`reward_asset_invariant_violation` 仅由 `reward_runtime.invariant_ok=false` 触发。
- 新参数落地后若二进制未重编译，脚本会在启动期失败。
  - 缓解：将 `cargo build -p agent_world --bin world_chain_runtime` 纳入回归步骤。
