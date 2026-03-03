# Agent World: 测试覆盖与 CI 扩展（项目管理）

## 任务拆解（含 PRD-ID 映射）
- [x] T1 (PRD-TESTING-CI-COVER-001/002): 新增离线回放 viewer 联测（snapshot/journal -> server -> client）。
- [x] T1.1 (PRD-TESTING-CI-COVER-002): 补充 viewer server 单次运行入口（run_once）。
- [x] T2 (PRD-TESTING-CI-COVER-001/003): CI 增加 wasmtime 特性测试步骤。
- [x] T3 (PRD-TESTING-CI-COVER-001/002): 文档更新（联测运行方式与 CI 覆盖说明）。
- [x] T4 (PRD-TESTING-CI-COVER-003): CI 安装 viewer 系统依赖（Wayland/X11/ALSA/UDev）。
- [x] T5 (PRD-TESTING-CI-COVER-001): 统一 CI/预提交测试清单脚本（`scripts/ci-tests.sh`）。
- [x] T6 (PRD-TESTING-CI-COVER-001): CI 与 pre-commit 接入统一脚本。
- [x] T7 (PRD-TESTING-CI-COVER-001): 文档同步（ci-test-coverage/pre-commit/visualization）。
- [x] T8 (PRD-TESTING-CI-COVER-003): `egui` snapshot 测试在 `wgpu` 不可用时自动 skip。
- [x] T9 (PRD-TESTING-CI-COVER-001): 移除默认 CI/pre-commit 的 builtin wasm hash 校验（改为手动触发）。
- [x] T10 (PRD-TESTING-004): 专题文档人工迁移到 strict schema，并切换命名为 `.prd.md/.prd.project.md`。

## 依赖
- `ViewerServer` / `ViewerServerConfig`
- `world_viewer_demo` 回放数据
- CI workflow 配置
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
