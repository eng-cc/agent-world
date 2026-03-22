# Close Beta Runtime Evidence (2026-03-22)

审计轮次: 3

## Meta
- 责任角色: `runtime_engineer`
- 目标: 为 `closed_beta_candidate` 场景收集 runtime 侧 five-node no-LLM soak / replay/rollback / release gate 证据，并确认脚本默认链路是否具备可重复执行的清理闭环。
- 主执行命令: `timeout 130 ./scripts/s10-five-node-game-soak.sh --duration-secs 120 --out-dir output/longrun/closed-beta-20260322 --no-prewarm --no-llm`
- 诊断复跑 1: `timeout 80 ./scripts/s10-five-node-game-soak.sh --duration-secs 60 --out-dir output/longrun/closed-beta-repro-20260322 --no-prewarm --no-llm`
- 诊断复跑 2: `timeout 140 ./scripts/s10-five-node-game-soak.sh --duration-secs 120 --out-dir output/longrun/closed-beta-repro-20260322 --no-prewarm --no-llm`
- cleanup 验证: `./scripts/s10-five-node-game-soak.sh --duration-secs 10 --base-port 6410 --out-dir output/longrun/closed-beta-cleanup-check-20260322 --no-prewarm --no-llm`
- 当前结果: `block`
- 关键产物:
  - 失败样本 A: `output/longrun/closed-beta-20260322/20260322-113809/{summary.md,summary.json,timeline.csv,nodes/*,failures.md}`
  - 通过样本 B: `output/longrun/closed-beta-repro-20260322/20260322-115646/{summary.md,summary.json,timeline.csv,nodes/*}`
  - 失败样本 C: `output/longrun/closed-beta-repro-20260322/20260322-115830/{summary.md,summary.json,nodes/s10-sequencer/stderr.log,failures.md}`
  - cleanup 验证 D/E: `output/longrun/closed-beta-cleanup-check-20260322/{20260322-120244,20260322-120255}/`

## 结论
- 当前 `TASK-GAME-029` 仍然阻断，但阻断原因已从“reward runtime 根本不可用”收敛为“两类独立问题叠加”：一是 sequencer 退出后脚本把后续观测写成全零次生失败；二是脚本 cleanup 不彻底时，下一轮会直接被默认端口残留 listener 阻断。
- 证据分层如下：
  - 失败样本 A `output/longrun/closed-beta-20260322/20260322-113809` 在 42 秒左右出现 `process_exit`，`metric_gate=fail`；但 sequencer 备份目录里实际已生成 `reward-runtime-report/epoch-0.json` 等产物，说明最初的 `minted_records_empty` / `reward_runtime_metrics_not_ready` 不是根因，而是进程退出后的次生告警。
  - 通过样本 B `output/longrun/closed-beta-repro-20260322/20260322-115646` 跑满 60 秒，`process_status=ok`、`metric_gate=pass`、`minted_non_empty_samples=5`、`reward_runtime_available_samples=5`，证明 five-node no-LLM 链路本身可跑通。
  - 失败样本 C `output/longrun/closed-beta-repro-20260322/20260322-115830` 在 startup 阶段直接报 `bind 127.0.0.1:5811 failed: Address already in use`，将问题进一步收敛到脚本 cleanup。
  - 已修复 `scripts/s10-five-node-game-soak.sh`，增加针对当前 run 的 `node-id + status-bind + node-gossip-bind` 精确 cleanup 与端口释放等待；cleanup 验证 D/E 在 `--base-port 6410` 下实现两轮 back-to-back 启动/收尾，无 `Address already in use`、无 startup fail、无 lingering process。
- 因此，当前还不能把默认端口的失败样本当作“runtime 内核必现不稳定”的最终签名，也不能把 60 秒/10 秒诊断样本当成 `closed_beta_candidate` 的正式放行证据；正式 120 秒默认端口复验、600 秒候选样本与 replay/rollback drill 仍未完成。

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
- 说明:
  - `node_state_backup/*` 是脚本启动前通过 `isolate_node_state_dirs()` 挪走的旧状态，但失败样本 A 的 sequencer 备份目录确实证明 reward runtime 在退出前已经生成过有效产物，因此原文档里“奖励/结算证据完全不存在”的表述需要废弃。
  - cleanup 验证是为脚本修复服务的诊断样本，不能替代 candidate soak。

## 阻断
- 当前 `closed_beta_candidate` gate 阻断点:
  1. 默认端口组 `5811-5815/5831-5835` 尚未在 cleanup 修复后重新拿到正式 120 秒通过样本，因此当前 candidate 长跑证据仍不可信。
  2. `closed_beta_candidate` 需要同一候选版本下的正式 600 秒 soak、replay/rollback drill 与 release gate 汇总；现有通过样本仍只是诊断 run。
  3. 在统一 release gate 收样前，仍需避免并发使用同一组 `output/chain-runtime` / `output/node-distfs` 节点目录，以免污染证据目录与结论口径。

## 下一步
1. 在默认端口链路上重新执行 120 秒 soak，确认 cleanup 修复后不再出现 `Address already in use` 或 lingering listener。
2. 在无并发 run 的前提下补齐 `600s+` candidate soak 与 replay/rollback drill。
3. 只有当正式候选样本在修复后的默认链路上拿到 `metric_gate=pass`，且 replay/rollback 证据齐全后，才能把 `summary.json/md`、`timeline.csv` 交回 `qa_engineer` 进入统一 `closed_beta_candidate` release gate。
