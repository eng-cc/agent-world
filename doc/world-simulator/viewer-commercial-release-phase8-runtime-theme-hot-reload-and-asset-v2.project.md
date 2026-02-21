# Viewer 商业化发行缺口收敛 Phase 8（项目管理）

## 任务拆解
- [x] VCR8-0 文档建档：设计文档 + 项目管理文档
- [x] VCR8-1 运行时主题状态、preset 解析与应用引擎
- [ ] VCR8-2 右侧面板主题控制区与热重载流程（含测试）
- [ ] VCR8-3 `industrial_v2` 资产包与预设落地
- [ ] VCR8-4 主题资产校验脚本与 smoke 验证
- [ ] VCR8-5 手册、状态回写、devlog 与测试收口

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/main.rs`
- `scripts/generate-viewer-industrial-theme-assets.py`
- `doc/viewer-manual.md`

## 状态
- 当前阶段：VCR8-1 已完成，VCR8-2 进行中。
- 下一步：接入右侧面板主题控制区并补热重载交互测试。
- 最近更新：2026-02-21（VCR8-1 完成并进入 VCR8-2）。
