# Close Beta Runtime Evidence (2026-03-22)

审计轮次: 2

## Meta
- 责任角色: `runtime_engineer`
- 目标: 为 `closed_beta_candidate` 场景收集 runtime 侧 five-node no-LLM soak / replay/rollback / release gate 证据。
- 主执行命令: `timeout 130 ./scripts/s10-five-node-game-soak.sh --duration-secs 120 --out-dir output/longrun/closed-beta-20260322 --no-prewarm --no-llm`
- 诊断复跑 1: `timeout 90 ./scripts/s10-five-node-game-soak.sh --duration-secs 60 --base-port 5910 --out-dir output/longrun/closed-beta-20260322-rerun2 --no-prewarm --no-llm`
- 诊断复跑 2: `timeout 90 ./scripts/s10-five-node-game-soak.sh --duration-secs 60 --out-dir output/longrun/closed-beta-20260322-rerun3 --no-prewarm --no-llm`
- 当前结果: `block`
- 关键产物:
  - 失败样本: `output/longrun/closed-beta-20260322/20260322-113809/{summary.md,summary.json,timeline.csv,nodes/*,failures.md}`
  - 通过样本（隔离端口）: `output/longrun/closed-beta-20260322-rerun2/20260322-115920/{summary.md,summary.json,timeline.csv,nodes/*}`
  - 失败样本（默认端口复现）: `output/longrun/closed-beta-20260322-rerun3/20260322-120053/{summary.md,summary.json,timeline.csv,nodes/*,failures.md}`

## 结论
- 当前 `TASK-GAME-029` 仍然阻断，但阻断原因已从“runtime 明确 panic/崩溃”收敛为“默认端口组环境不可信，导致候选门禁样本被外部 SIGTERM/脏资源污染”。
- 证据分层如下：
  - 默认端口首轮样本 `output/longrun/closed-beta-20260322/20260322-113809` 在 42 秒左右出现 `process_exit`，`metric_gate=fail`，当时脚本还拿不到子进程退出码。
  - 补充脚本观测后，隔离端口样本 `output/longrun/closed-beta-20260322-rerun2/20260322-115920` 在 `--base-port 5910` 下跑满 60 秒，`process_status=ok`、`metric_gate=pass`、`minted_non_empty_samples=5`、`reward_runtime_available_samples=5`，说明 runtime 与 reward runtime 在干净端口环境下可以完成最小长跑闭环。
  - 回到默认端口的复现实验 `output/longrun/closed-beta-20260322-rerun3/20260322-120053` 在约 8 秒时再次失败，`node=s10-sequencer exited during soak exit_status=143`，stderr 为空；`143` 对应 `SIGTERM`，更符合外部清理/端口环境污染，而不是 runtime 自身 panic。
- 因此，当前还不能把默认端口的失败样本当作“runtime 内核不稳定”的最终签名，也不能把隔离端口 60 秒样本当成 `closed_beta_candidate` 的正式放行证据；正式 600 秒候选样本与 replay/rollback drill 仍未完成。

## 门禁与指标
- 首轮失败样本 `20260322-113809`:
  - `metric_gate=fail`
  - `running_false_samples=5`
  - `minted_non_empty_samples=0`
  - `reward_runtime_available_samples=0`
  - `committed_height_monotonic=false`
- 隔离端口通过样本 `20260322-115920`:
  - `metric_gate=pass`
  - `status_samples_ok=120`
  - `balances_samples_ok=120`
  - `minted_non_empty_samples=5`
  - `settlement_positive_samples=1`
  - `reward_runtime_available_samples=5`
  - `committed_height_monotonic=true`
- 默认端口复现失败样本 `20260322-120053`:
  - `process_exit` with `exit_status=143`
  - `metric_gate=fail`
  - `minted_non_empty_samples=0`
  - `reward_runtime_available_samples=0`
  - `http_failure_samples=2`
- 说明:
  - `node_state_backup/*` 是脚本启动前通过 `isolate_node_state_dirs()` 挪走的历史节点状态，不能拿来证明本轮失败样本已经生成 reward runtime report。
  - 当前脚本已新增 `nodes/<node>/exit-status.txt`，后续再遇到 `process_exit` 时可以区分 `exit code` 与 `signal`。

## 阻断
- 当前 `closed_beta_candidate` gate 阻断点:
  1. 默认端口组 `5811-5815/5831-5835` 的样本不稳定，已两次拿到失败证据，其中一次明确记录为 `exit_status=143`；在端口环境可信前，当前候选长跑证据不具备发布门禁效力。
  2. `closed_beta_candidate` 需要同一候选版本下的正式 600 秒 soak、replay/rollback drill 与 release gate 汇总；现有通过样本只有 60 秒隔离端口诊断 run，不能直接关闭 `TASK-GAME-029`。
  3. 当前并发环境里还存在其他 `s10-five-node-game-soak.sh` 使用同一组 `output/chain-runtime` / `output/node-distfs` 节点目录的风险，继续并发跑批会污染证据目录与结论口径。

## 下一步
1. 固化 clean-room 运行条件：要么为 `TASK-GAME-029` 保留专用 `base_port`，要么在启动前增加端口占用/残留进程探测，避免默认端口被外部流程抢占或回收。
2. 在无并发 run 的前提下重新执行候选样本，优先补 `120s -> 600s` 的同版本 five-node no-LLM soak，并同步跑 replay/rollback drill。
3. 只有当正式候选样本在可信端口环境下拿到 `metric_gate=pass`，且 replay/rollback 证据齐全后，才能把 `summary.json/md`、`timeline.csv` 交回 `qa_engineer` 进入统一 `closed_beta_candidate` release gate。
