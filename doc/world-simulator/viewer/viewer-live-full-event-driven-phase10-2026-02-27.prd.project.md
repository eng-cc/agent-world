# Viewer Live 完全事件驱动改造 Phase 10（项目管理）

审计轮次: 2

## ROUND-002 物理合并
- 本文件为主项目入口（当前权威入口）。
- `phase8/phase9` 项目内容已物理合并入本文件，对应阶段项目文档降级为历史追溯。
- 历史阶段项目文档:
  - `doc/world-simulator/archive/viewer-live-full-event-driven-phase8-2026-02-27.prd.project.md`
  - `doc/world-simulator/archive/viewer-live-full-event-driven-phase9-2026-02-27.prd.project.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 代码收敛：`viewer/server` 删除 `tick_interval` 与定时回放推进
- [x] T2 代码收敛：`viewer/web_bridge` 删除 `poll_interval` 与轮询 sleep
- [x] T3 收口：活跃手册/入口示例与测试同步清理
- [x] T4 回归与结项：required 测试 + 阶段文档收口

## Phase 8/9 增量任务记录（已合并）

### Phase 8
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 代码收敛：删除 script 回退开关与 `timer_pulse` 回退链路，仅保留 event_drive
- [x] T2 测试改造：更新/清理回退模式断言并完成 required 回归
- [x] T3 文档收口：更新阶段结论与遗留事项

### Phase 9
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 代码收敛：删除 `world_viewer_live --tick-ms` 与 `CliOptions.tick_ms`
- [x] T2 脚本收敛：删除所有脚本对 `--tick-ms` 的透传链路
- [x] T3 文档收敛：更新活跃手册与脚本说明，移除 `--tick-ms` 示例
- [x] T4 回归与结项：required 测试 + 阶段文档收口

## 依赖
- doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.md
- `crates/agent_world/src/viewer/server.rs`
- `crates/agent_world/src/viewer/web_bridge.rs`
- `crates/agent_world/tests/viewer_offline_integration.rs`
- `site/index.html`
- `site/en/index.html`
- `testing-manual.md` / `doc/world-simulator/viewer/viewer-manual.md`（如需）
- Phase 8 补充依赖: `doc/world-simulator/archive/viewer-live-full-event-driven-phase8-2026-02-27.prd.md`、`crates/agent_world/src/viewer/live_split_part1.rs`、`crates/agent_world/src/viewer/live_split_part2.rs`、`crates/agent_world/src/viewer/live/tests.rs`、`crates/agent_world/src/viewer/mod.rs`、`crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
- Phase 9 补充依赖: `doc/world-simulator/archive/viewer-live-full-event-driven-phase9-2026-02-27.prd.md`、`crates/agent_world/src/bin/world_viewer_live/cli.rs`、`crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part2.rs`、`crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests_split_part1.rs`、`crates/agent_world/tests/viewer_live_integration.rs`、`scripts/capture-viewer-frame.sh`、`scripts/viewer-theme-pack-preview.sh`、`scripts/run-game-test.sh`、`scripts/p2p-longrun-soak.sh`、`scripts/viewer-owr4-stress.sh`、`scripts/viewer-release-qa-loop.sh`、`scripts/viewer-texture-inspector.sh`、`testing-manual.md`、`doc/world-simulator/viewer/viewer-manual.md`

## 状态
- 当前阶段：已完成（T0~T4）
- 备注：Phase 8/9 任务记录已合并归档，阶段项目文档转为历史追溯。
