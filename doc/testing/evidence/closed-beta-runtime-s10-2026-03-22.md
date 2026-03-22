# Close Beta Runtime Evidence (2026-03-22)

审计轮次: 4

## Meta
- 责任角色: `runtime_engineer`
- 目标: 为 `closed_beta_candidate` 场景收集 runtime 侧 five-node no-LLM soak / replay/rollback / release gate 证据，并确认脚本默认链路是否具备可重复执行的清理闭环。
- 主执行命令: `timeout 130 ./scripts/s10-five-node-game-soak.sh --duration-secs 120 --out-dir output/longrun/closed-beta-20260322 --no-prewarm --no-llm`
- 诊断复跑 1: `timeout 80 ./scripts/s10-five-node-game-soak.sh --duration-secs 60 --out-dir output/longrun/closed-beta-repro-20260322 --no-prewarm --no-llm`
- 诊断复跑 2: `timeout 140 ./scripts/s10-five-node-game-soak.sh --duration-secs 120 --out-dir output/longrun/closed-beta-repro-20260322 --no-prewarm --no-llm`
- cleanup 验证: `./scripts/s10-five-node-game-soak.sh --duration-secs 10 --base-port 6410 --out-dir output/longrun/closed-beta-cleanup-check-20260322 --no-prewarm --no-llm`
- clean-room 候选复验: `./scripts/s10-five-node-game-soak.sh --duration-secs 120 --base-port 6310 --out-dir output/longrun/closed-beta-rerun-fixed-20260322 --no-prewarm --no-llm`
- 当前结果: `block`
- 关键产物:
  - 失败样本 A: `output/longrun/closed-beta-20260322/20260322-113809/{summary.md,summary.json,timeline.csv,nodes/*,failures.md}`
  - 通过样本 B: `output/longrun/closed-beta-repro-20260322/20260322-115646/{summary.md,summary.json,timeline.csv,nodes/*}`
  - 失败样本 C: `output/longrun/closed-beta-repro-20260322/20260322-115830/{summary.md,summary.json,nodes/s10-sequencer/stderr.log,failures.md}`
  - cleanup 验证 D/E: `output/longrun/closed-beta-cleanup-check-20260322/{20260322-120244,20260322-120255}/`
  - 通过样本 F: `output/longrun/closed-beta-rerun-fixed-20260322/20260322-120458/{summary.md,summary.json,timeline.csv,nodes/*}`

## 结论
- 当前 `TASK-GAME-029` 仍然阻断，但阻断原因已从“reward runtime 根本不可用”收敛为“两类独立问题叠加”：一是旧脚本在 sequencer 退出后继续把后续观测写成全零次生失败；二是脚本 cleanup 不彻底时，下一轮会被残留 listener / 脏端口环境直接阻断。
- 证据分层如下：
  - 失败样本 A `output/longrun/closed-beta-20260322/20260322-113809` 在 42 秒左右出现 `process_exit`，`metric_gate=fail`；但 sequencer 备份目录里实际已生成 `reward-runtime-report/epoch-0.json` 等产物，说明最初的 `minted_records_empty` / `reward_runtime_metrics_not_ready` 不是根因，而是进程退出后的次生告警。
  - 通过样本 B `output/longrun/closed-beta-repro-20260322/20260322-115646` 跑满 60 秒，`process_status=ok`、`metric_gate=pass`、`minted_non_empty_samples=5`、`reward_runtime_available_samples=5`，证明 five-node no-LLM 链路本身可跑通。
  - 失败样本 C `output/longrun/closed-beta-repro-20260322/20260322-115830` 在 startup 阶段直接报 `bind 127.0.0.1:5811 failed: Address already in use`，将问题进一步收敛到脚本 cleanup。
  - 已修复 `scripts/s10-five-node-game-soak.sh`，增加针对当前 run 的 `node-id + status-bind + node-gossip-bind` 精确 cleanup 与端口释放等待；cleanup 验证 D/E 在 `--base-port 6410` 下实现两轮 back-to-back 启动/收尾，无 `Address already in use`、无 startup fail、无 lingering process。
  - 通过样本 F `output/longrun/closed-beta-rerun-fixed-20260322/20260322-120458` 在 clean-room 端口组 `6310` 下跑满 120 秒，`process_status=ok`、`metric_gate=pass`、`minted_non_empty_samples=120`、`reward_runtime_available_samples=120`、`settlement_apply_attempts=2`，且 `nodes/*/exit-status.txt` 全部是脚本收尾阶段写入的 `exit_status=143/signal=15`，证明当前退出签名属于受控 `SIGTERM` 停机而不是 runtime crash。
- 因此，当前已不能再把“120 秒候选样本未补齐”作为 runtime lane 的正式阻断；剩余缺口收敛为：还缺同一候选版本下的 `600s+` soak 与 replay/rollback drill，才能形成完整 `closed_beta_candidate` runtime 硬证据包。

## 门禁与指标
- 失败样本 A `20260322-113809`:
  - `metric_gate=fail`
  - `running_false_samples=5`
  - `minted_non_empty_samples=0`
  - `reward_runtime_available_samples=0`
  - `committed_height_monotonic=false`
- 通过样本 B `20260322-115646`:
  - `metric_gate=pass`
  - `status_samples_ok=120`
  - `balances_samples_ok=120`
  - `minted_non_empty_samples=5`
  - `settlement_positive_samples=1`
  - `reward_runtime_available_samples=5`
  - `committed_height_monotonic=true`
- 失败样本 C `20260322-115830`:
  - `startup_failed`
  - `metric_gate=fail`
  - sequencer stderr: `bind 127.0.0.1:5811 failed: Address already in use`
- cleanup 验证 D/E:
  - 两轮都 `process_status=ok`
  - 两轮结束后 `ps -ef | rg 'oasis7_chain_runtime --node-id s10-.*(641|643)'` 无残留
- 通过样本 F `20260322-120458`:
  - `process_status=ok`
  - `metric_gate=pass`
  - `minted_non_empty_samples=120`
  - `settlement_positive_samples=24`
  - `reward_runtime_available_samples=120`
  - `settlement_apply_attempts=2`
  - `committed_height_monotonic=true`
- 说明:
  - `node_state_backup/*` 是脚本启动前通过 `isolate_node_state_dirs()` 挪走的旧状态，但失败样本 A 的 sequencer 备份目录确实证明 reward runtime 在退出前已经生成过有效产物，因此原文档里“奖励/结算证据完全不存在”的表述需要废弃。
  - cleanup 验证 D/E 是为脚本修复服务的短诊断样本，不能替代 candidate soak；样本 F 才是当前最新的 clean-room 120 秒候选证据。

## 阻断
- 当前 `closed_beta_candidate` gate 阻断点:
  1. `closed_beta_candidate` 仍需要同一候选版本下的正式 `600s+` soak、replay/rollback drill 与 release gate 汇总；当前最新通过样本只覆盖 clean-room 120 秒。
  2. 在统一 release gate 收样前，仍需避免并发复用同一组节点目录或端口组，以免再次污染候选证据目录与结论口径。
  3. QA gate、viewer、pure API、no-UI smoke 还未在同一 candidate 上完成 fresh rerun，因此 runtime lane 即使补齐 120 秒也不能单独推动阶段升级。

## 下一步
1. 沿用 clean-room 端口策略补齐同一候选版本下的 `600s+` soak，避免共享默认端口组再次污染运行环境。
2. 在同一 candidate 上补齐 replay/rollback drill，并将命令、stdout/stderr、artifact 目录与 soak 结果挂到同一证据包。
3. 只有当 `600s+` soak 与 replay/rollback 证据齐全后，才能把 `summary.json/md`、`timeline.csv` 与 drill 记录交回 `qa_engineer` 进入统一 `closed_beta_candidate` release gate。
