# OpenClaw 双轨模式 T4 阻断记录（2026-03-16）

- owner: `qa_engineer`
- 关联 PRD: `PRD-WORLD_SIMULATOR-040`
- 关联任务: `doc/world-simulator/llm/llm-openclaw-agent-dual-mode-2026-03-16.project.md` T4
- 联审建议: `producer_system_designer`、`agent_engineer`、`runtime_engineer`、`viewer_engineer`
- 文档状态: `resolved`（2026-03-17，`TASK-WORLD_SIMULATOR-152`）

## 1. 结论
- `TASK-WORLD_SIMULATOR-152` 已把真实 `player_parity` 执行 lane 接到 runtime live / launcher / parity bench / `oasis7`。
- 当前已经能分别对 `headless_agent` 与 `player_parity` 路径给出真实 OpenClaw smoke 通过证据。
- T4 不再被“`player_parity` 未接线”阻断；当前状态改为 `pending / ready_for_qa`，等待 `qa_engineer` / `producer_system_designer` 基于真实双样本给出正式对照结论。

## 2. 已验证证据
### 2.1 环境准备
- `openclaw --version` 返回 `OpenClaw 2026.3.13 (61d171a)`。
- `curl -sS http://127.0.0.1:18789/health` 返回 `{"ok":true,"status":"live"}`。
- `.agents/skills/oasis7/scripts/oasis7-run.sh doctor --json` 初次检查显示 Gateway 正常，但 `http://127.0.0.1:5841` bridge 未启动。

### 2.2 headless_agent 真实 smoke
- 历史执行：`.agents/skills/oasis7/scripts/oasis7-run.sh smoke --samples 1 --ticks 4 --timeout-ms 15000`
- 历史结果：自动拉起 bridge 后，`openclaw_local_http` smoke 通过。
- 历史产物：
  - `artifacts/openclaw_parity_20260316_235632/summary/P0-001.openclaw_local_http.json`
  - `artifacts/openclaw_parity_20260316_235632/samples/openclaw_local_http/sample_1/summary/P0-001.openclaw_local_http.json`
  - `artifacts/openclaw_parity_20260316_235632/samples/openclaw_local_http/sample_1/raw/P0-001_sample_1.openclaw_local_http.jsonl`
- 历史关键指标：
  - `status=passed`
  - `goal_completed=1`
  - `decision_steps=4`
  - `invalid_action_count=0`
  - `timeout_count=0`
  - `trace_completeness_ratio_ppm=1000000`

### 2.3 2026-03-17 双模式复验
- headless 执行：`.agents/skills/oasis7/scripts/oasis7-run.sh smoke --samples 1 --ticks 4 --timeout-ms 15000 --execution-mode headless_agent`
- player parity 执行：`.agents/skills/oasis7/scripts/oasis7-run.sh smoke --samples 1 --ticks 4 --timeout-ms 15000 --execution-mode player_parity`
- headless 产物：
  - `artifacts/openclaw_parity_20260317_002147/summary/P0-001.openclaw_local_http.json`
  - `artifacts/openclaw_parity_20260317_002147/samples/openclaw_local_http/sample_1/summary/P0-001.openclaw_local_http.json`
- player parity 产物：
  - `artifacts/openclaw_parity_20260317_002217/summary/P0-001.openclaw_local_http.json`
  - `artifacts/openclaw_parity_20260317_002217/samples/openclaw_local_http/sample_1/summary/P0-001.openclaw_local_http.json`
- 两条链路共同指标：
  - `status=passed`
  - `goal_completed=1`
  - `decision_steps=4`
  - `invalid_action_count=0`
  - `timeout_count=0`
  - `trace_completeness_ratio_ppm=1000000`

## 3. 已解除的根因
### 3.1 代码修复点
以下位置现在已经支持真实 `player_parity` lane：
- `crates/agent_world/src/viewer/runtime_live/llm_sidecar.rs`：通过 `AGENT_WORLD_OPENCLAW_EXECUTION_MODE` 解析并透传 runtime live OpenClaw execution mode。
- `crates/agent_world/src/bin/world_game_launcher.rs`：新增 `--openclaw-execution-mode`，把 execution mode 透传给 `world_viewer_live`。
- `crates/agent_world/src/bin/world_openclaw_parity_bench.rs`：新增 `--execution-mode`，真实 OpenClaw parity bench 不再固定为 `headless_agent`。
- `scripts/openclaw-parity-p0.sh` 与 `.agents/skills/oasis7/scripts/oasis7-run.sh`：新增 execution mode 参数并贯通 smoke / play 操作路径。

### 3.2 对 T4 的当前影响
- “缺少真实 `player_parity` lane”这一代码阻断已经解除。
- T4 现在具备正式执行前提，剩余工作是由 `qa_engineer` / `producer_system_designer` 基于真实双模式样本给出默认模式与阻断结论。

## 4. QA 建议
- 当前口径：
  - `headless_agent`：继续作为无 GUI / 无 GPU 回归主链路。
  - `debug_viewer`：继续作为旁路观战/解释层。
  - `player_parity`：已具备真实运行 lane，可进入正式对照采证。
- 下一个必需动作：由 `qa_engineer` / `producer_system_designer` 在同一 OpenClaw 场景上重跑并对比 `player_parity` vs `headless_agent`，回写 T4 默认模式与阻断结论。
