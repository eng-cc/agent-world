# Agent World：P2P/存储/共识在线长跑稳定性测试方案（2026-02-24）

## 目标
- 建立一套可重复执行的长跑测试，验证系统在持续在线运行时的稳定性，而非只验证短时功能正确。
- 覆盖 P2P 网络、分布式存储（DistFS challenge/probe）、共识推进三条核心链路，并给出可审计证据。
- 将结果分级为长跑专用档位 `soak_smoke` 与 `soak_endurance`，支持日常高风险改动回归与发布前长稳验收。

## 范围

### In Scope
- 新增长跑测试编排脚本（暂定：`scripts/p2p-longrun-soak.sh`）。
- 复用 `world_viewer_live` 现有分布式拓扑与报表能力：
  - `--topology triad`、`--topology triad_distributed`
  - `--reward-runtime-enable`
  - `--reward-runtime-report-dir`
- 采集并聚合运行证据：进程存活、共识高度推进、网络追平、存储挑战成功率、不变量审计状态。
- 新增故障注入（受控）用于检验恢复能力：节点重启、短时断连、挑战压力抬升。
- 输出统一产物目录与失败报告模板，支持 devlog 与发布验收复盘。

### Out of Scope
- 跨物理机/跨地域部署压测编排（本轮先做单机多进程闭环）。
- 新共识算法或新存储协议语义改造。
- UI/Web 交互验证（已有 S6/S8 覆盖）。

## 接口 / 数据

### 1) 长跑入口脚本（草案）
- `./scripts/p2p-longrun-soak.sh [options]`
- 关键参数（草案）：
  - `--profile <soak_smoke|soak_endurance|soak_release>`
  - `--duration-secs <n>`
  - `--topology <triad|triad_distributed>`
  - `--scenario <name>`（默认 `triad_p2p_bootstrap`）
  - `--tick-ms <n>`
  - `--enable-chaos`
  - `--chaos-plan <path>`
  - `--out-dir <path>`（默认 `.tmp/p2p_longrun`）
  - `--max-stall-secs <n>`
  - `--max-lag-p95 <n>`
  - `--max-distfs-failure-ratio <0~1>`

### 2) 分档策略（长跑专用命名）
- `soak_smoke`（长跑冒烟档）
  - 推荐：`20~30` 分钟，`triad` + `triad_distributed` 各 1 轮。
  - 用途：高风险改动本地/合并前快速判断“是否出现明显长稳退化”。
- `soak_endurance`（发布回归档）
  - 推荐：`180` 分钟（基础），可扩展到 `8~24` 小时（发布前）。
  - 用途：验证长时累计状态、恢复路径、指标漂移与资源边界。
- `soak_release`（发行验收档）
  - 用途：在 `soak_endurance` 基础上叠加更严格阈值与完整 chaos 计划。

### 2.1) 命名边界（避免与 feature 语义混淆）
- `soak_*` 仅表示“长跑测试执行档位”，不等同于 Cargo feature、`test_tier_required`、`test_tier_full` 或 CI required/full。
- 如需联动现有分层，文档只做“执行建议映射”，不复用同名。

### 3) 拓扑与负载矩阵（草案）
- Case A：`triad`（单进程三角色内嵌）
  - 目标：快速发现共识推进/执行桥接/挑战探测退化。
- Case B：`triad_distributed`（三进程：sequencer/storage/observer）
  - 目标：验证真实网络路径下的 head 传播、追平与复制行为。
- Case C：`triad_distributed + chaos`
  - 目标：验证节点重启与短时故障下的恢复能力与追平时延。
- 运行期统一开启：
  - `--reward-runtime-enable`
  - DistFS probe/adaptive 参数按 `soak_*` profile 提供默认值并允许覆盖。

### 4) 证据源（已存在能力）
- `world_viewer_live` epoch 报表 JSON（`--reward-runtime-report-dir`）：
  - `node_snapshot.consensus.{committed_height,network_committed_height,peer_heads,last_status,last_execution_height}`
  - `distfs_challenge_report.{total_checks,passed_checks,failed_checks,failure_reasons}`
  - `reward_asset_invariant_status.ok`
  - 代码依据：`crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part2.rs`。
- 进程级指标：
  - PID 存活、退出码、重启次数。
  - RSS 周期采样（`ps`）用于内存泄漏趋势观察。
- 日志证据：
  - 每节点 stdout/stderr。
  - 故障注入事件时间线。

### 5) 核心通过标准（草案）
- 稳定性
  - 进程无崩溃退出（允许受控 chaos 重启）。
  - 报表连续产出（无长时间中断）。
- 共识
  - `committed_height` 单调不回退。
  - 连续无推进窗口不超过 `max_stall_secs`。
  - `network_committed_height - committed_height` 的 p95 不超过阈值。
- 存储挑战
  - `failed_checks / total_checks` 不超过阈值（在 `total_checks>0` 条件下）。
  - `failure_reasons` 不出现持续单一原因爆发（按连续窗口判定）。
- 资产与结算不变量
  - `reward_asset_invariant_status.ok == true` 全程成立。
- 资源趋势
  - RSS 增长斜率不超过 `soak_*` profile 阈值；若超阈值，标记为失败或高危告警。

### 6) 产物目录（草案）
- `.tmp/p2p_longrun/<timestamp>/`
  - `run_config.json`
  - `timeline.csv`
  - `summary.json`
  - `summary.md`
  - `failures.md`（失败时）
  - `chaos_events.log`
  - `nodes/<node_id>/{stdout.log,stderr.log,rss.csv,report/*.json}`

### 7) 与测试手册/CI 的接线（草案）
- `testing-manual.md` 新增 `S9`：Non-Viewer 分布式长跑套件。
- `S9` 触发建议：
  - 任何触达 `agent_world_net` / `agent_world_node` / `agent_world_consensus` / `agent_world_distfs` 的高风险改动。
- CI 策略（分步推进）：
  - 阶段 1：仅手动与发布前执行。
  - 阶段 2：夜间 `workflow_dispatch/schedule` 跑 `soak_endurance` 长跑档。

## 里程碑
- M0：方案建档（设计文档 + 项目管理文档）。
- M1：实现编排脚本最小闭环（启动/停止/超时/产物目录）。
- M2：实现指标聚合与门禁判定（summary + failures）。
- M3：实现故障注入与恢复验证（chaos plan）。
- M4：接入 `testing-manual.md`（S9）并形成执行剧本。
- M5：完成一次 `soak_smoke` 与一次 `soak_endurance` 实跑留档。

## 风险
- 指标口径风险：不同拓扑下报表频率不一致，可能造成误判。
  - 缓解：先做“必需字段 + 时间窗口容错”，再逐步收紧阈值。
- 环境噪声风险：单机资源竞争会放大抖动。
  - 缓解：记录机器资源上下文，阈值区分 `soak_smoke`/`soak_endurance` 档。
- 长跑耗时风险：全量长跑不适合每次提交。
  - 缓解：保留分档执行，`soak_smoke` 用于冒烟，`soak_endurance`/`soak_release` 用于夜间与发布前。
- 误报风险：chaos 注入与真实故障日志可能混淆。
  - 缓解：统一 `chaos_events.log` 时间线，判定器按注入窗口豁免。

## 当前状态（2026-02-24）
- M0：已完成。
- M1~M5：未开始。
