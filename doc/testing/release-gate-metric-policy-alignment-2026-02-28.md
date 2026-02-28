# 发布门禁指标策略对齐（2026-02-28）

## 目标
- 解决 S9/S10 发布门禁在 `world_chain_runtime` 链路下的误杀问题，使“进程稳定 + 核心共识健康”可直接放行。
- 保留严格门禁能力：需要时可显式开启“指标严格模式”拦截 `insufficient_data` 与空 mint 样本。

## 范围
- 脚本改造：
  - `scripts/p2p-longrun-soak.sh`
  - `scripts/s10-five-node-game-soak.sh`
- 运行手册同步：
  - `testing-manual.md`（补充默认/严格口径）

## 非目标
- 不修改 `world_chain_runtime` 运行时实现。
- 不新增 epoch 报表字段，仅调整脚本门禁策略与参数开关。

## 接口 / 数据
- 新增可选严格模式参数（默认关闭）：
  - S9：`--strict-metrics`
  - S10：`--strict-metrics`
- 默认策略（`--strict-metrics` 未启用）：
  - `minted_records_empty` 记为告警，不再作为 S10 硬失败。
  - `metric_gate=insufficient_data` 仅记注释，不在 S9 `soak_release` 强制失败。
- 严格策略（启用 `--strict-metrics`）：
  - `minted_records_empty` 作为硬失败。
  - `metric_gate=insufficient_data` 在 S9/S10 都升级为失败（若进程状态为 `ok`，则转 `metric_gate_failed`）。
- 产物兼容：
  - `run_config.json` 增加 `strict_metrics` 字段；
  - 维持原有 `summary.json`/`summary.md`/`failures.md` 结构与路径。

## 里程碑
- M1：建档（设计 + 项目管理）。
- M2：脚本实现严格开关并调整默认门禁策略。
- M3：复跑 S9/S10 发布门禁命令并确认通过。
- M4：手册与项目文档收口，补任务日志。

## 风险
- 默认放宽后，可能放过低价值采样窗口。
  - 缓解：保留 `--strict-metrics`，在 RC 冻结/高风险改动时启用。
- 现网调用脚本可能依赖旧的失败语义。
  - 缓解：help 与手册显式标注默认行为变化，保留严格模式开关。
