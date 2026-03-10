# Viewer 可视化基础架构设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-visualization.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-visualization.project.md`

## 1. 设计定位
定义 Agent World Viewer 从协议、回放、UI 到 3D 基础能力的总体可视化架构：为离线回放、在线 live、时间轴、事件联动和基础调试能力提供统一底座。

## 2. 设计结构
- 协议与回放层：定义 hello/subscribe/snapshot/event/control 与 offline/live 回放路径。
- Viewer 运行层：Bevy viewer crate 承担窗口、输入、网络连接和 headless/offline 模式。
- UI 观察层：世界状态、事件浏览、回放控制、指标、诊断和对象联动组成右侧面板主路径。
- 3D 表达层：背景参照、覆盖层、时间轴与对象选择形成基础可视化闭环。

## 3. 关键接口 / 入口
- viewer 协议消息结构
- `world_viewer_server` / `world_viewer_live`
- `agent_world_viewer`
- offline/live/headless 模式入口

## 4. 约束与边界
- 这是总体基础架构文档，不替代后续专题设计文档。
- 协议、UI、3D 和测试必须协同演进，不能各自为政。
- 历史能力可以演进，但总文档需要保持当前权威入口。
- 基础可视化架构强调信息直达，不等于默认暴露所有调试噪声。

## 5. 设计演进计划
- 先落 viewer 协议、server 和 Bevy viewer 基础链路。
- 再扩时间轴、对象联动、覆盖层和诊断。
- 最后用 headless/offline/live 测试矩阵冻结底座能力。
