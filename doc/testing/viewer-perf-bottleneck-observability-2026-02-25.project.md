# Viewer 性能瓶颈观测能力补齐（项目管理文档）

## 任务拆解
- [x] T1：建档（设计文档 + 项目管理文档）。
- [ ] T2：扩展 `RenderPerfSummary` 与 `PerfHotspot` 推断逻辑（含单测）。
- [ ] T3：扩展 `perf_probe` 与 `scripts/viewer-owr4-stress.sh` 输出/解析（含脚本回归）。
- [ ] T4：更新相关测试、回写文档状态与 devlog。

## 依赖
- `crates/agent_world_viewer/src/render_perf_summary.rs`
- `crates/agent_world_viewer/src/perf_probe.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `scripts/viewer-owr4-stress.sh`
- `testing-manual.md`
- `doc/performance_testing_manual(viewer).md`
- `doc/devlog/2026-02-25.md`

## 状态
- 当前阶段：T1 完成，T2 进行中。
- 阻塞项：无。
