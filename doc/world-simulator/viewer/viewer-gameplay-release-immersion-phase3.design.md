# Viewer 发行体验沉浸改造 Phase 3 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase3.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase3.project.md`

## 1. 设计定位
定义第三阶段的情绪闭环与世界活性方案：在 Player 模式中补上成就反馈与 Agent 事件气泡，让玩家感知“我做成了什么”和“世界正在回应我”。

## 2. 设计结构
- 成就反馈层：围绕连接、首次事件、首次选中等里程碑维护成就解锁集合与弹层队列。
- 世界活性层：把增量事件转成 Agent 气泡短句，增强世界说话感。
- 统一渲染层：HUD、引导、toast、成就和气泡共用 Player 体验渲染入口。
- 防噪约束层：首帧不回放历史事件，队列长度和淡出时长受控。

## 3. 关键接口 / 入口
- `egui_right_panel_player_experience.rs`
- 成就解锁集合 / 队列
- Agent 气泡队列与增量事件游标
- `ViewerExperienceMode`

## 4. 约束与边界
- 只增强 Player 体验层，不改仿真协议与大型美术资源。
- 反馈必须短句、高价值、受限流控制，不能挤占主场景。
- Director 模式保持调试导向，不强制展示成就和气泡。
- 事件到文案的映射需要保持可测试，避免语义漂移。

## 5. 设计演进计划
- 先落成就状态与弹层队列。
- 再补 Agent 气泡与统一渲染整合。
- 最后用单测和 Web 闭环验证情绪闭环。
