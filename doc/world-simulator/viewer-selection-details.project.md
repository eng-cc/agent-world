# Viewer 选中对象详情面板（含 LLM 决策 I/O）（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-simulator/viewer-selection-details.md`）
- [x] 输出项目管理文档（本文件）
- [x] 扩展 LLM 决策 trace 数据结构与行为接口
- [x] live viewer 协议新增 `DecisionTrace` 消息并在 LLM 模式推送
- [x] viewer 新增“选中对象详情”面板（Agent/Location 分支）
- [x] 详情面板接入最近事件与最近 LLM trace 展示
- [x] LLM 诊断字段扩展（模型/耗时/token/重试）
- [x] 选中对象扩展：Asset/PowerPlant/PowerStorage（3D marker + 点击 + 详情）
- [x] 选中对象扩展：Chunk（3D marker + 点击 + 详情）
- [x] 新增/更新测试（协议 round-trip + viewer UI 文案 + live trace 流）
- [x] 新增/更新测试（Asset/PowerPlant 详情文案）
- [x] 事件对象联动补齐：`ModuleVisualEntityUpserted/Removed` 映射到可点击对象
- [x] 右侧信息区增加滚动条与滚轮滚动（长内容可浏览）
- [x] 新增/更新测试（模块可视事件映射、滚动行为）
- [x] 更新可视化文档与总项目管理文档状态
- [x] 更新任务日志并提交

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/viewer/protocol.rs`
- `crates/agent_world/src/viewer/live.rs`
- `crates/agent_world_viewer/src/main.rs`

## 状态
- 当前阶段：M6（模块可视联动 + 面板滚动）完成
- 下一阶段：按需增加滚动区域粒度控制（分区独立滚动）
- 最近更新：完成 M6（模块可视事件联动 + 右侧滚动条 + 回归测试，2026-02-07）
