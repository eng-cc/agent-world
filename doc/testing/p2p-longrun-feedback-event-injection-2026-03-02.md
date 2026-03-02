# Agent World：P2P 长跑反馈事件注入（2026-03-02）

## 目标
- 在多节点长程脚本 `scripts/p2p-longrun-soak.sh` 中加入“反馈提交事件”注入能力，覆盖 `POST /v1/chain/feedback/submit` 真实链路。
- 让长跑不仅验证共识/存储/奖励指标，还能持续施加真实业务事件流量（玩家 bug/建议反馈）。
- 输出可审计证据（配置、注入日志、成功/失败统计），便于回归对比和故障定位。

## 范围

### In Scope
- 扩展 `scripts/p2p-longrun-soak.sh`：新增 feedback 注入参数、注入执行器、统计与产物字段。
- 支持反馈注入与现有 chaos（plan/continuous）并行运行。
- 在 `run_config.json` / `summary.json` / `summary.md` 增加 feedback 相关配置与统计。
- 更新 `testing-manual.md` S9 命令示例与通过标准说明。

### Out of Scope
- 不改造 `s10-five-node-game-soak.sh`（本轮先落地在 S9 主脚本）。
- 不新增反馈 append/tombstone 注入（本轮仅 create）。
- 不调整 node/runtime 共识或 DistFS 语义。

## 接口 / 数据

### 新增 CLI（草案）
- `--feedback-events-enable`
- `--feedback-events-interval-secs <n>`（默认 60）
- `--feedback-events-start-sec <n>`（默认 30）
- `--feedback-events-max-events <n>`（默认 0，表示直到拓扑结束）

### 注入策略（草案）
- 从 `start-sec` 开始，按 `interval-secs` 周期注入。
- 目标节点按轮询选择，类别在 `bug/suggestion` 间交替。
- 提交失败计入失败计数并写入注入日志；不中断主循环（避免单次抖动打断整场长跑）。

### 证据与统计（草案）
- 新产物：`feedback_events.log`
- `summary.json` 每拓扑新增：
  - `feedback_events`
  - `feedback_events_success`
  - `feedback_events_failed`
- `summary.json` totals 新增：
  - `feedback_events_total`
  - `feedback_events_success_total`
  - `feedback_events_failed_total`
- `summary.md` 增加反馈注入配置与计数展示。

## 里程碑
- M1：设计/项目文档建档。
- M2：脚本参数与注入执行器落地。
- M3：summary/run_config/日志证据扩展。
- M4：S9 手册接线与回归验证。

## 风险
- 风险：反馈接口短时抖动导致失败计数偏高。
  - 缓解：失败不打断主流程；独立统计成功/失败，结合 gate 结果判断。
- 风险：注入频率过高干扰主业务指标。
  - 缓解：默认保守参数（60s）并允许显式上限。
- 风险：脚本复杂度继续上升。
  - 缓解：新增逻辑保持单独函数与独立日志，避免侵入已有 gate 计算路径。

## 完成态（2026-03-02）
- `scripts/p2p-longrun-soak.sh` 已接入 feedback 事件注入参数、执行器、日志与 summary 汇总字段。
- `testing-manual.md` S9 已补充 feedback 注入命令与通过标准。
- 项目管理文档已收口为“已完成（F0~F3）”。
