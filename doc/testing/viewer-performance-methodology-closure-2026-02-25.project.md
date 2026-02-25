# Viewer 性能测试方法论能力补齐（项目管理文档）

## 任务拆解
- [x] T1：输出设计文档与项目管理文档（建档）。
- [x] T2：增强 `scripts/viewer-owr4-stress.sh`（自动采集 + 门禁判定 + 基线对比 + 输出结构化结果）。
- [x] T3：执行脚本验证（语法、help、短窗冒烟）并更新文档状态与 devlog。
- [x] T4：补充 Web 启动测试文档中的 GPU 前置要求（渲染性能采样必须 GPU + headed）。

## 依赖
- `scripts/viewer-owr4-stress.sh`
- `crates/agent_world_viewer/src/perf_probe.rs`
- `doc/testing/viewer-performance-methodology-closure-2026-02-25.md`
- `testing-manual.md`
- `doc/performance_testing_manual(viewer).md`
- `doc/devlog/2026-02-25.md`

## 状态
- 当前阶段：T1/T2/T3/T4 已完成。
- 阻塞项：无。
