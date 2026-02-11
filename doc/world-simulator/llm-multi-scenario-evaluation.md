# Agent World Simulator：LLM 多场景评测基线（设计文档）

## 目标
- 解决仅使用 `llm_bootstrap` 单场景评测导致的样本偏差问题，建立可复用的多场景评测基线。
- 在不破坏现有压测脚本使用习惯的前提下，支持批量运行多个场景并输出聚合指标。
- 为后续 LMSO29.x / LMSO30 优化提供统一验收口径（稳定性 + 效率 + 动作成效）。

## 范围
- In Scope：
  - 扩展 `scripts/llm-longrun-stress.sh`，支持多次 `--scenario` 与 `--scenarios <csv>`。
  - 产出“按场景分目录”结果（`report.json` / `run.log` / `summary.txt`）。
  - 产出跨场景聚合 `report.json` 与 `summary.txt`（总量、均值、峰值、阈值判定）。
  - 更新 README 与项目管理文档，给出多场景运行示例。
- Out of Scope：
  - 不修改 `world_llm_agent_demo` 的业务逻辑与 trace 数据结构。
  - 不新增世界场景文件，仅复用已有 scenario。
  - 不引入新的评测服务端或数据库存储。

## 接口 / 数据
- 输入接口（脚本参数）：
  - `--scenario <name>`：可重复传入多个场景。
  - `--scenarios <csv>`：逗号分隔场景列表。
  - `--jobs <n>`：多场景并行度（默认 `1`，即串行）。
  - 兼容原有单场景参数（未指定时默认 `llm_bootstrap`）。
- 输出数据：
  - 单场景：保持现有输出路径语义（向后兼容）。
  - 多场景：
    - `<out-dir>/scenarios/<scenario>/report.json`
    - `<out-dir>/scenarios/<scenario>/run.log`
    - `<out-dir>/scenarios/<scenario>/summary.txt`
    - `<out-dir>/report.json`（聚合）
    - `<out-dir>/summary.txt`（聚合）
    - `<out-dir>/run.log`（聚合日志）
- 聚合指标（首版）：
  - 稳定性：`llm_errors_total`、`parse_errors_total`、`repair_rounds_max_peak`。
  - 效率：`llm_input_chars_total`、`llm_input_chars_avg_mean`、`llm_input_chars_max_peak`、`module_call_total`。
  - 行动：`action_success_total`、`action_failure_total`、`decision_act_total`、`decision_wait_total`。

## 里程碑
- M1：完成脚本参数扩展与多场景执行流程。
- M2：完成聚合结果落盘（json + summary）与阈值判定策略。
- M3：完成 README / 项目文档 / devlog 回写，并运行一次 30 tick 多场景验证。

## 风险
- **运行时长上涨**：多场景批量会显著增加时长；通过 `--ticks` 与场景子集控制首轮验证成本。
- **场景差异放大波动**：不同地形与资源初始化会带来更高指标方差；通过同时输出“分场景明细 + 聚合”降低误判。
- **兼容风险**：保留单场景默认行为，避免影响现有 CI 与手工使用习惯。

## 已实施与验证（2026-02-11）
- 已实施：
  - `scripts/llm-longrun-stress.sh` 已支持多次 `--scenario` 与 `--scenarios <csv>` 混用。
  - `scripts/llm-longrun-stress.sh` 已支持 `--jobs <n>` 多场景并行执行（默认 `1`）。
  - 多场景模式已落盘“分场景明细 + 聚合总览”：
    - `<out-dir>/scenarios/<scenario>/report.json|run.log|summary.txt`
    - `<out-dir>/report.json|run.log|summary.txt`
  - `README.md` 已补充多场景运行示例与输出路径说明。
- 验证命令：
  - `./scripts/llm-longrun-stress.sh --ticks 30 --scenarios llm_bootstrap,power_bootstrap,resource_bootstrap --out-dir .tmp/lmso30_multi_30 --no-llm-io --max-parse-errors 5 --max-repair-rounds-max 5`
- 聚合结果（`.tmp/lmso30_multi_30/summary.txt`）：
  - `action_success_total=86`，`action_failure_total=1`
  - `llm_errors_total=0`，`parse_errors_total=3`
  - `llm_input_chars_avg_mean=1399`，`llm_input_chars_max_peak=20680`
  - `module_call_total=8`，`plan_total=16`

## 回归观察（偏差确认）
- 单场景存在明显偏差：仅看 `llm_bootstrap` 会遗漏 `power_bootstrap/resource_bootstrap` 中出现的 `parse_errors` 波动。
- 场景间输入峰值差异显著：
  - `llm_bootstrap`：`llm_input_chars_max=20680`
  - `power_bootstrap`：`llm_input_chars_max=12300`
  - `resource_bootstrap`：`llm_input_chars_max=12287`
- 结论：后续验收不应只用单场景；至少保留 3 场景聚合口径，必要时扩展至 5 场景。

## 补充验证（5 场景，2026-02-11）
- 命令：
  - `./scripts/llm-longrun-stress.sh --ticks 30 --scenarios llm_bootstrap,power_bootstrap,resource_bootstrap,twin_region_bootstrap,triad_region_bootstrap --out-dir .tmp/lmso30_multi_5scenes_30 --no-llm-io --max-parse-errors 5 --max-repair-rounds-max 5`
- 聚合结果（`.tmp/lmso30_multi_5scenes_30/summary.txt`）：
  - `action_success_total=140`，`action_failure_total=9`
  - `llm_errors_total=0`，`parse_errors_total=0`
  - `llm_input_chars_avg_mean=2072`，`llm_input_chars_max_peak=19584`
  - `module_call_total=29`，`plan_total=32`
- 分场景摘要：
  - `llm_bootstrap`：`action_success=26`，`action_failure=4`，`llm_input_chars_avg=3275`
  - `power_bootstrap`：`action_success=30`，`action_failure=0`，`llm_input_chars_avg=1209`
  - `resource_bootstrap`：`action_success=30`，`action_failure=0`，`llm_input_chars_avg=1224`
  - `twin_region_bootstrap`：`action_success=27`，`action_failure=2`，`llm_input_chars_avg=2721`
  - `triad_region_bootstrap`：`action_success=27`，`action_failure=3`，`llm_input_chars_avg=1931`
- 观察：
  - 3 场景与 5 场景在 `parse_errors_total` 上出现差异（`3 -> 0`），说明单次采样有随机性，必须采用多场景 + 多轮样本看趋势，而非单次结论。

## 并行能力验证（2026-02-11）
- 命令（冒烟）：
  - `./scripts/llm-longrun-stress.sh --ticks 1 --scenarios llm_bootstrap,power_bootstrap --jobs 2 --out-dir .tmp/lmso30_parallel_smoke --no-llm-io --max-parse-errors 5 --max-repair-rounds-max 5`
- 结果：
  - 并行模式可正常完成并输出聚合结果（`mode=multi_scenario`，`jobs=2`）。
  - 聚合 `report.json` 已包含 `jobs` 字段，便于后续横向对比不同并行度运行。

## 并行全量验证（2026-02-11）
- 命令（5 场景并行）：
  - `./scripts/llm-longrun-stress.sh --ticks 30 --scenarios llm_bootstrap,power_bootstrap,resource_bootstrap,twin_region_bootstrap,triad_region_bootstrap --jobs 5 --out-dir .tmp/lmso30_multi_5scenes_30_jobs5 --no-llm-io --max-parse-errors 5 --max-repair-rounds-max 5`
- 聚合结果（`.tmp/lmso30_multi_5scenes_30_jobs5/summary.txt`）：
  - `action_success_total=138`，`action_failure_total=7`
  - `llm_errors_total=0`，`parse_errors_total=4`
  - `llm_input_chars_avg_mean=1948`，`llm_input_chars_max_peak=20903`
  - `module_call_total=21`，`plan_total=35`
- 观察：
  - 并行模式可用于全量评测闭环；当前主要优化点仍在 `resource_bootstrap/twin_region_bootstrap` 的 parse 波动。

## 后续优化点
- 增设“稳定性硬门槛”分层判定：
  - 聚合门槛：`llm_errors_total==0`
  - 单场景门槛：每个场景 `parse_errors <= 1`（或按阶段放宽再回收）。
- 引入固定评测场景集（建议 5 场景）：`llm_bootstrap,power_bootstrap,resource_bootstrap,twin_region_bootstrap,triad_region_bootstrap`。
- 在聚合报告中补充分位统计（如 `llm_input_chars_avg` 的 P50/P95），减少均值掩盖峰值风险。
