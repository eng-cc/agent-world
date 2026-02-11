# Viewer 面向开放世界沙盒可玩化准备（项目管理文档）

## 任务拆解

### OWR0 文档与对齐
- [x] OWR0.1 输出设计文档（`doc/world-simulator/viewer-open-world-sandbox-readiness.md`）
- [x] OWR0.2 输出项目管理文档（本文件）
- [x] OWR0.3 在总项目文档挂载分册入口并同步阶段状态

### OWR1 观察态/运营态信息架构（对应缺口 1）
- [x] OWR1.1 定义 `Observe/Prompt Ops` 双模式 UI 状态机与切换规则
- [x] OWR1.2 新增运营态总览（Prompt-only 约束、Agent 选择、Prompt 草稿区、审计占位）
- [x] OWR1.3 补充 UI 回归测试（模式文案、Prompt Ops 面板渲染）并通过 `cargo test -p agent_world_viewer`

### OWR2 Prompt 控制面（对应缺口 2，严格 prompt-only）
- [ ] OWR2.1 协议扩展：`prompt_control.preview/apply/rollback`
- [ ] OWR2.2 live server 接入 `AgentPromptProfile`（版本、审计、冲突处理）
- [ ] OWR2.3 事件扩展：`AgentPromptUpdated`（用于时间轴与回放）
- [ ] OWR2.4 viewer 右侧编辑区：差异预览、提交、回滚、错误提示
- [ ] OWR2.5 前后端联测：preview/apply/rollback 成功与失败路径
- [ ] OWR2.6 回放一致性测试：prompt 变更后 replay 对齐

### OWR3 3D 表达与性能收口（对应缺口 4）
- [x] OWR3.1 完成 VPP5：选中反馈强化（缩放 + 视觉强调）
- [x] OWR3.2 完成 VPP6：标签 LOD（距离衰减、数量上限、遮挡降权）
- [ ] OWR3.3 完成 VPP7：覆盖层/网格线批处理与节流
- [ ] OWR3.4 完成 VPP8：右侧总览接入渲染性能摘要
- [ ] OWR3.5 完成 VPP9：2D/3D 与联动全回归

### OWR4 规模化稳定性（对应缺口 5）
- [ ] OWR4.1 事件窗口策略（滚动窗口 + 采样）与配置项
- [ ] OWR4.2 场景对象增量刷新（脏区更新）与回归测试
- [ ] OWR4.3 自动降级策略（标签/覆盖层分级关闭）
- [ ] OWR4.4 压测脚本与报告模板（triad/llm 高负载）
- [ ] OWR4.5 形成跨版本基线（帧时间、事件吞吐、丢帧/卡顿指标）

### OWR5 收口
- [x] OWR5.1 更新设计文档与项目文档状态
- [x] OWR5.2 更新任务日志（`doc/devlog/YYYY-MM-DD.md`）
- [x] OWR5.3 执行 `env -u RUSTC_WRAPPER cargo check` 与相关测试
- [ ] OWR5.4 按任务粒度提交 git commit

## 依赖
- 协议与服务端：`crates/agent_world/src/viewer/protocol.rs`、`crates/agent_world/src/viewer/live.rs`
- viewer：`crates/agent_world_viewer/src/main.rs`、`egui_right_panel.rs`、`scene_helpers.rs`、`world_overlay.rs`
- 既有 3D 优化基线：`doc/world-simulator/viewer-3d-polish-performance*.md`
- 联测与脚本：`crates/agent_world/tests/viewer_live_integration.rs`、`scripts/capture-viewer-frame.sh`

## 状态
- 当前阶段：OWR1 已完成（Observe/Prompt Ops 双模式 UI 骨架已落地）
- 下一阶段：OWR2（prompt_control 协议与 `AgentPromptUpdated` 事件链路）
- 最近更新：完成 Prompt Ops 面板落地、`AGENT_WORLD_VIEWER_PANEL_MODE` 启动模式支持、单测与截图闭环验证（2026-02-11）

## 不在本轮
- TODO-3：动作玩法闭环（发现-采集-加工-建造）。
- TODO-6：产品化能力（音频、资产管线、新手引导等）。
