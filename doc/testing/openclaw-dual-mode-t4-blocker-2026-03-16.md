# OpenClaw 双轨模式 T4 阻断记录（2026-03-16）

- owner: `qa_engineer`
- 关联 PRD: `PRD-WORLD_SIMULATOR-040`
- 关联任务: `doc/world-simulator/llm/llm-openclaw-agent-dual-mode-2026-03-16.project.md` T4
- 联审建议: `producer_system_designer`、`agent_engineer`、`runtime_engineer`、`viewer_engineer`

## 1. 结论
- 当前只能对 `headless_agent` 路径给出真实 OpenClaw smoke 通过证据。
- 当前不能对 `player_parity` vs `headless_agent` 给出正式对照结论，T4 保持 `pending / blocked`。
- 阻断原因不是 OpenClaw Gateway / bridge 不可用，而是产品代码里 `player_parity` 执行 lane 尚未接入可运行链路。

## 2. 已验证证据
### 2.1 环境准备
- `openclaw --version` 返回 `OpenClaw 2026.3.13 (61d171a)`。
- `curl -sS http://127.0.0.1:18789/health` 返回 `{"ok":true,"status":"live"}`。
- `.agents/skills/oasis7/scripts/oasis7-run.sh doctor --json` 初次检查显示 Gateway 正常，但 `http://127.0.0.1:5841` bridge 未启动。

### 2.2 headless_agent 真实 smoke
- 执行：`.agents/skills/oasis7/scripts/oasis7-run.sh smoke --samples 1 --ticks 4 --timeout-ms 15000`
- 结果：自动拉起 bridge 后，`openclaw_local_http` smoke 通过。
- 关键产物：
  - `artifacts/openclaw_parity_20260316_235632/summary/P0-001.openclaw_local_http.json`
  - `artifacts/openclaw_parity_20260316_235632/samples/openclaw_local_http/sample_1/summary/P0-001.openclaw_local_http.json`
  - `artifacts/openclaw_parity_20260316_235632/samples/openclaw_local_http/sample_1/raw/P0-001_sample_1.openclaw_local_http.jsonl`
- 关键指标：
  - `status=passed`
  - `goal_completed=1`
  - `decision_steps=4`
  - `invalid_action_count=0`
  - `timeout_count=0`
  - `trace_completeness_ratio_ppm=1000000`

## 3. 当前阻断点
### 3.1 player_parity 未接入产品执行链路
以下位置表明当前产品/bench/runtime live 只把真实 OpenClaw 接到 `headless_agent`：
- `crates/agent_world/src/viewer/runtime_live/llm_sidecar.rs`：runtime live OpenClaw runner 固定 `.with_execution_mode(ProviderExecutionMode::HeadlessAgent)`。
- `crates/agent_world/src/bin/world_openclaw_parity_bench.rs`：真实 OpenClaw parity bench 固定 `.with_execution_mode(ProviderExecutionMode::HeadlessAgent)`。
- 代码库中 `ProviderExecutionMode::PlayerParity` 当前只出现在 enum/测试，没有对应产品执行路径接线。

### 3.2 对 T4 的影响
- 缺少真实 `player_parity` lane，就无法在“同一 OpenClaw 场景、同一 provider、同一 task 口径”下产出 `player_parity` vs `headless_agent` 对照证据。
- 因此当前不能满足 `PRD-WORLD_SIMULATOR-040` T4 对“默认模式与阻断结论”的正式验收要求。

## 4. QA 建议
- 维持当前结论：
  - `headless_agent`：可继续作为无 GUI / 无 GPU 回归主链路。
  - `debug_viewer`：可继续作为旁路观战/解释层。
  - `player_parity`：在真实运行 lane 落地前，不得宣称已完成对照采证。
- 下一个必需动作：由 `agent_engineer` / `runtime_engineer` 先把真实 `player_parity` 执行路径接到 provider/bench/runtime live 之一，再由 `qa_engineer` / `producer_system_designer` 重跑 T4。
