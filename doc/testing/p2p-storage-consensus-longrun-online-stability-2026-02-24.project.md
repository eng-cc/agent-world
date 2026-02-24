# Agent World：P2P/存储/共识在线长跑稳定性测试（项目管理文档）

## 任务拆解
- [x] T0：完成方案建档
  - [x] 设计文档：`doc/testing/p2p-storage-consensus-longrun-online-stability-2026-02-24.md`
  - [x] 项目管理文档：`doc/testing/p2p-storage-consensus-longrun-online-stability-2026-02-24.project.md`
- [ ] T1：实现长跑编排脚本最小闭环
  - [x] 新增 `scripts/p2p-longrun-soak.sh`（启动/停止/超时/清理/目录结构）
  - [x] 支持 `triad` 与 `triad_distributed` 两种拓扑
- [ ] T2：实现指标采样与聚合判定
  - [ ] 解析 `reward_runtime_report_dir` 下 epoch JSON
  - [ ] 输出 `timeline.csv`、`summary.json`、`summary.md`
  - [ ] 实现门禁：stall/lag/distfs failure ratio/invariant ok
- [ ] T3：实现故障注入与恢复验证
  - [ ] 支持重启/短时断连注入计划（`--chaos-plan`）
  - [ ] 输出 `chaos_events.log` 并与判定器联动
- [ ] T4：文档与手册接线
  - [ ] 在 `testing-manual.md` 新增 S9 套件
  - [ ] 新增 S9 执行剧本、证据规范、触发矩阵条目
- [ ] T5：验证与收口
  - [ ] 执行一次 `soak_smoke` 长跑档
  - [ ] 执行一次 `soak_endurance` 长跑档
  - [ ] 回写 `doc/devlog/2026-02-24.md` 与状态收口

## 依赖
- `testing-manual.md`
- `scripts/ci-tests.sh`
- `scripts/viewer-owr4-stress.sh`
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part2.rs`
- `crates/agent_world/src/bin/world_viewer_live/distfs_probe_runtime.rs`
- `crates/agent_world/src/bin/world_viewer_live/distfs_challenge_network.rs`
- `crates/agent_world_node/src/types.rs`

## 状态
- 当前阶段：T0/T1 已完成，T2 待开始。
- 阻塞项：无。
- 下一步：执行 T2（补指标采样、聚合判定与 summary 结构化输出）。
- 最近更新：2026-02-24。
