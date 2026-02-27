# Viewer Live 完全事件驱动改造 Phase 9（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 代码收敛：删除 `world_viewer_live --tick-ms` 与 `CliOptions.tick_ms`
- [ ] T2 脚本收敛：删除所有脚本对 `--tick-ms` 的透传链路
- [ ] T3 文档收敛：更新活跃手册与脚本说明，移除 `--tick-ms` 示例
- [ ] T4 回归与结项：required 测试 + 阶段文档收口

## 依赖
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part2.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests_split_part1.rs`
- `scripts/capture-viewer-frame.sh`
- `scripts/viewer-theme-pack-preview.sh`
- `scripts/run-game-test.sh`
- `scripts/p2p-longrun-soak.sh`
- `scripts/viewer-owr4-stress.sh`
- `scripts/viewer-release-qa-loop.sh`
- `scripts/viewer-texture-inspector.sh`
- `testing-manual.md`
- `doc/viewer-manual.md`

## 状态
- 当前阶段：进行中（T0~T1 已完成，进行 T2）
- 备注：Phase 9 只清理 viewer live 旧 tick 入口，不改 node/runtime 基础 tick 机制。
