# Viewer Web 可操作性硬化设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-web-usability-hardening-2026-02-22.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-web-usability-hardening-2026-02-22.project.md`

## 1. 设计定位
定义 Web 端窄屏布局、连接错误与自动重连的综合硬化方案：在不改协议的前提下提升可操作性和舒适度。

## 2. 设计结构
- 窄屏布局层：根据 available width 和模块状态启用紧凑模式与宽度预算。
- 错误回调层：把 websocket `onerror` 处理从宽泛假设改成安全提取。
- 连接恢复层：增加带退避的自动重连状态和用户友好提示。
- 标准化文案层：技术错误转换为用户可理解描述，同时保留原始兜底。

## 3. 关键接口 / 入口
- 窄屏模式判定与宽度预算
- websocket `onerror` 安全处理
- 自动重连状态
- 错误文案标准化方法

## 4. 约束与边界
- 只做 Web 端可用性加固，不改主题风格和协议字段。
- 聊天降级到主面板内后仍要保持可访问。
- 自动重连要有退避和最大间隔，避免频繁抖动。
- 标准化文案不能完全吞掉原始技术错误。

## 5. 设计演进计划
- 先补窄屏布局。
- 再修 onerror 类型问题。
- 最后接自动重连与连接行为回归。
