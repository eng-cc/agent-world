# Viewer 启动自动选中（用于截图闭环）（项目管理文档）

## 任务拆解
- [x] ASC-1：输出设计文档（`doc/world-simulator/viewer-auto-select-capture.md`）
- [x] ASC-2：输出项目管理文档（本文件）
- [ ] ASC-3：viewer 新增自动选中配置与系统（agent/location）
- [ ] ASC-4：补充/更新测试（配置解析 + 选中行为）
- [ ] ASC-5：脚本新增 `--auto-select-target` 参数与环境变量透传
- [ ] ASC-6：执行截图闭环验证并更新日志

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/auto_select.rs`（新增）
- `scripts/capture-viewer-frame.sh`

## 状态
- 当前阶段：ASC-2 完成，进入 ASC-3。
- 下一阶段：完成代码、测试与截图闭环后收口。
- 最近更新：新增自动选中设计与任务拆解（2026-02-11）。
