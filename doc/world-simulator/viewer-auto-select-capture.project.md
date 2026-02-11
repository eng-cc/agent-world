# Viewer 启动自动化步骤（相机/选中，用于截图闭环）（项目管理文档）

## 任务拆解
- [x] ASC-1：输出设计文档（`doc/world-simulator/viewer-auto-select-capture.md`）
- [x] ASC-2：输出项目管理文档（本文件）
- [x] ASC-3：viewer 新增自动化配置与执行系统（mode/focus/pan/zoom/orbit/select/wait）
- [x] ASC-4：补充/更新测试（自动化解析 + 默认模式拆分测试）
- [x] ASC-5：脚本新增 `--auto-select-target`、`--automation-steps` 参数与环境变量透传
- [x] ASC-6：执行截图闭环验证并更新日志

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/viewer_automation.rs`（新增）
- `crates/agent_world_viewer/src/ui_state_types.rs`（新增）
- `crates/agent_world_viewer/src/tests_camera_mode.rs`（新增）
- `scripts/capture-viewer-frame.sh`

## 状态
- 当前阶段：ASC-6 完成，任务收口。
- 下一阶段：按需扩展 `select` 目标类型（asset/power/chunk）和自动化回放模板。
- 最近更新：完成启动自动化步骤能力（相机 + 选中）与闭环验证（2026-02-11）。
