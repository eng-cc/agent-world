# Viewer 首局目标与控制语义可解释反馈优化（2026-02-27）

## 目标
- 提升首局可理解性：首屏/首阶段明确展示 `1 个主目标 + 2 个短目标`，降低“目标模糊”。
- 提升控制可发现性：为 `sendControl` 提供内置动作字典、payload 示例和可直接填充入口，降低“盲试协议”。
- 提升输入可解释性：每次输入返回结构化反馈（是否接受、失败原因、解析结果、建议下一步），减少“系统没听懂”的不确定感。

## 范围

### In Scope
- `crates/agent_world_viewer` 玩家引导 HUD 文案与结构改造。
- Web Test API 新增动作描述接口与 `sendControl` 结构化返回。
- 控制输入反馈状态纳入 `getState` 输出，便于自动化与人工复盘。
- 对应 viewer 单元测试补充（test_tier_required）。

### Out of Scope
- 不改 runtime 业务规则与 action 执行语义。
- 不改 LLM 策略生成逻辑。
- 不做新的玩法系统，仅优化首局引导与控制语义可解释性。

## 接口/数据
- 玩家目标文案：
  - 现有 `PlayerGuideStep` 扩展为“主目标 + 两个短目标”渲染结构。
- Web Test API：
  - 新增 `describeControls(): object`，返回动作清单、payload schema、示例。
  - `sendControl(action, payload): object` 返回结构化结果：
    - `accepted: bool`
    - `action: string`
    - `parsedControl: string | null`
    - `reason: string | null`
    - `hint: string | null`
- `getState()` 扩展：
  - 新增 `lastControlFeedback`，用于显示最近控制输入反馈摘要。

## 里程碑
- M1：完成文档建档与任务拆解。
- M2：完成首局主/短目标 UI 改造与测试。
- M3：完成控制语义可发现 + 输入可解释反馈改造与测试。
- M4：完成验证、项目文档收口与日志沉淀。

## 风险
- Web Test API 在 wasm 场景下返回对象结构新增字段，可能影响旧自动化脚本断言。
- 引导文案改造若与现有玩家模式布局冲突，可能造成 UI 拥挤。
- 过多调试信息可能干扰普通玩家视图，需要在呈现层保持克制。
