# Viewer 商业化发行缺口收敛 Phase 8（项目管理）

- 对应设计文档: `doc/world-simulator/viewer/viewer-commercial-release-phase8-runtime-theme-hot-reload-and-asset-v2.design.md`
- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase8-runtime-theme-hot-reload-and-asset-v2.prd.md`

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] VCR8-0 文档建档：设计文档 + 项目管理文档
- [x] VCR8-1 运行时主题状态、preset 解析与应用引擎
- [x] VCR8-2 右侧面板主题控制区与热重载流程（含测试）
- [x] VCR8-3 `industrial_v2` 资产包与预设落地
- [x] VCR8-4 主题资产校验脚本与 smoke 验证
- [x] VCR8-5 手册、状态回写、devlog 与测试收口

## 依赖
- doc/world-simulator/viewer/viewer-commercial-release-phase8-runtime-theme-hot-reload-and-asset-v2.prd.md
- `crates/oasis7_viewer/src/app_bootstrap.rs`
- `crates/oasis7_viewer/src/egui_right_panel.rs`
- `crates/oasis7_viewer/src/main.rs`
- `scripts/generate-viewer-industrial-theme-assets.py`
- `doc/world-simulator/viewer/viewer-manual.md`

## 状态
- 当前阶段：Phase 8 全部完成。
- 下一步：进入下一阶段商业发行缺口（UI polish/动效/发行验收）任务拆解。
- 最近更新：2026-02-21（VCR8-5 完成，Phase 8 收口）。
