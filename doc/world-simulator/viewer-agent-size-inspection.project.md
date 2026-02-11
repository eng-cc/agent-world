# Viewer Agent 尺寸可观测与比例校验（项目管理文档）

## 任务拆解
- [x] ASI-1：输出设计文档（`doc/world-simulator/viewer-agent-size-inspection.md`）
- [x] ASI-2：输出项目管理文档（本文件）
- [x] ASI-3：Selection Details（Agent）新增尺寸与比例字段
- [x] ASI-4：补充/更新测试（Agent 详情尺寸文案断言）
- [x] ASI-5：补充中文本地化映射并回归验证
- [x] ASI-6：更新开发日志并收口状态

## 依赖
- `crates/agent_world_viewer/src/ui_text.rs`
- `crates/agent_world_viewer/src/ui_locale_text.rs`
- `crates/agent_world_viewer/src/tests_selection_details.rs`

## 状态
- 当前阶段：ASI-6 已完成，任务收口。
- 下一阶段：按反馈决定是否补充“渲染尺寸（含 clamp 后尺寸）”与“Location 可视半径（depletion 后）”明细行。
- 最近更新：完成 Agent 尺寸/比例详情展示、i18n 映射与测试回归（2026-02-11）。
