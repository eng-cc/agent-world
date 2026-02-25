# Agent World：P2P 长跑 180 分钟 Chaos 模板方案（2026-02-25）

## 目标
- 提供一个可直接复用的“大规模 `chaos-plan` 模板”，用于 `soak_endurance`（180 分钟及以上）长跑稳定性验证。
- 与已有 continuous chaos 注入能力配合，形成“固定回归基线 + 连续高覆盖探索”的执行方式。
- 明确该模板属于 S9 长跑策略，不与 Cargo feature（`test_tier_required`/`test_tier_full`）语义混淆。

## 范围

### In Scope
- 新增仓库内可追踪的 `chaos-plan` 模板文件（覆盖 180 分钟窗口）。
- 模板采用分阶段注入策略，覆盖 `restart/pause/disconnect` 与多节点轮换。
- 更新 `testing-manual.md` S9，给出直接可跑命令与验收口径。
- 增加一次短窗可执行性验证（schema 与脚本兼容性）。

### Out of Scope
- 新增 chaos 动作类型（如磁盘打满、网络 tc 故障）。
- 修改 `scripts/p2p-longrun-soak.sh` 的执行语义。
- 跨机房或跨地域故障编排。

## 接口 / 数据

### 1) 模板文件
- 路径：`doc/testing/chaos-plans/p2p-soak-endurance-full-chaos-v1.json`
- JSON 顶层结构：
  - `meta`：版本、时长、注入策略说明。
  - `events`：事件数组（`id/at_sec/topology/node/action/down_secs/duration_secs`）。

### 2) 动作与节点约束
- `action`：`restart`、`pause`、`disconnect`。
- `topology=triad_distributed` 时节点限定：`sequencer`、`storage`、`observer`。
- 事件按 `at_sec` 递增，便于审计与排障。

### 3) 执行建议（S9）
- 固定计划：仅 `--chaos-plan <template>`。
- 混合探索：`--chaos-plan <template>` + `--chaos-continuous-enable`。
- 验收重点：`summary.json` 的 `chaos_plan_events_total`、`chaos_continuous_events_total`、`chaos_events_total` 与 `overall_status`。

## 里程碑
- M0：方案与项目管理文档建档。
- M1：提交 180 分钟 chaos-plan 模板文件。
- M2：S9 手册接线（命令示例与口径）。
- M3：短窗验证与收口留档。

## 风险
- 事件过密导致噪声过高，掩盖真实退化趋势。
  - 缓解：按阶段控制节奏（热身/稳态/扰动峰值/冷却）。
- 模板不可复现或难以比对。
  - 缓解：固定 `id` 与时间轴，作为回归基线。
- 与 test tier 语义混淆。
  - 缓解：文档显式声明 `soak_*` 为长跑执行档位，不等同 Cargo feature。

## 当前状态（2026-02-25）
- M0：已完成。
- M1：已完成（新增模板：`doc/testing/chaos-plans/p2p-soak-endurance-full-chaos-v1.json`，共 `190` 个事件）。
- M2：已完成（`testing-manual.md` S9 已接入 180 分钟模板命令，并补充 `soak_*` 与 `test_tier_*` 语义边界）。
- M3：未开始。
