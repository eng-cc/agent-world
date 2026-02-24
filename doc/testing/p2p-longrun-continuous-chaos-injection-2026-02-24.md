# Agent World：P2P 长跑持续 Chaos 注入方案（2026-02-24）

## 目标
- 在现有 `--chaos-plan` 固定注入能力上，新增“持续注入”能力，模拟真实线上长期抖动与故障组合。
- 支持“可复现 + 高覆盖”双模式：固定计划用于回归，连续注入用于探索未知组合。
- 保持与当前 S9 门禁兼容，输出可审计证据（注入时间线、计数、门禁结果）。

## 范围

### In Scope
- 扩展 `scripts/p2p-longrun-soak.sh`：支持持续 chaos 事件生成与执行。
- 支持与 `--chaos-plan` 并行工作（混合模式）。
- 支持可重复随机（seed）与注入节奏控制（interval/start/max-events）。
- 在 `run_config.json`/`summary.json`/`summary.md` 增加持续 chaos 配置与统计字段。
- 更新 `testing-manual.md` S9 使用说明。

### Out of Scope
- 新增内核级故障类型（如磁盘注满、网络层 tc 丢包）自动化注入。
- 跨主机/跨地域故障编排。
- 修改共识/存储协议语义。

## 接口 / 数据

### 1) 新增 CLI（草案）
- `--chaos-continuous-enable`
- `--chaos-continuous-interval-secs <n>`
- `--chaos-continuous-start-sec <n>`
- `--chaos-continuous-max-events <n>`（`0` 表示直到测试结束）
- `--chaos-continuous-actions <csv>`（`restart,pause,disconnect`）
- `--chaos-continuous-seed <n>`
- `--chaos-continuous-restart-down-secs <n>`
- `--chaos-continuous-pause-duration-secs <n>`

### 2) 调度规则（草案）
- 周期触发：从 `start-sec` 开始，每隔 `interval-secs` 触发一次。
- 动作与节点选择：在运行节点集合与 `actions` 集合中按 seed 伪随机选择。
- 安全约束：单次只执行一个事件（复用现有串行执行模型）；若事件失败，按现有 `chaos_failed` 处理。
- 与固定计划共存：固定计划与持续注入共享同一执行器与计数器。

### 3) 证据扩展（草案）
- `run_config.json`：记录持续注入开关与参数。
- `summary.json`：
  - 每拓扑：`chaos_plan_events`、`chaos_continuous_events`、`chaos_events`。
  - 全局 totals：`chaos_plan_events_total`、`chaos_continuous_events_total`、`chaos_events_total`。
- `summary.md`：补充 continuous chaos 配置与计数展示。

## 里程碑
- M0：方案与项目拆解建档。
- M1：脚本支持持续注入核心调度。
- M2：summary/run_config 证据扩展。
- M3：S9 手册接线。
- M4：完成一次持续注入短窗实跑留档。

## 风险
- 过度注入导致“人为打爆”而非真实退化。
  - 缓解：默认保守参数（较大 interval、短暂停时长）并暴露上限参数。
- 随机性导致不可复现。
  - 缓解：默认记录 seed；支持显式 seed 复跑。
- 固定计划与连续注入叠加过密。
  - 缓解：事件串行执行，失败即停止并出具 `chaos_events.log` + `failures.md`。

## 当前状态（2026-02-24）
- M0：已完成。
- M1：已完成（持续注入参数解析、校验、调度与执行链路已落地）。
- M2：已完成（run_config/summary 已补齐 continuous chaos 配置与 plan/continuous/total 计数字段）。
- M3：已完成（`testing-manual.md` S9 已接入 continuous chaos 示例与验收口径）。
- M4：未开始。
