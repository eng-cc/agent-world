# 启动器 Native/Web 控制面统一设计

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-native-web-control-plane-unification-2026-03-04.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-native-web-control-plane-unification-2026-03-04.project.md`

## 1. 设计定位
定义启动器 native 与 web 两端共用的状态模型、动作协议与日志/错误反馈闭环。

## 2. 设计结构
- 状态层：统一 launcher state、process state 与错误状态枚举。
- 动作层：统一 start/stop/query/retry 等控制面动作。
- 反馈层：native 与 web 共用日志、错误与恢复提示模型。

## 3. 关键接口 / 入口
- launcher 状态查询与控制 API
- UI action schema 与日志输出接口
- 进程编排与重试入口

## 4. 约束与边界
- Native/Web 不允许出现不同控制语义。
- 状态机迁移需保持幂等和可重试。
- 错误提示必须可映射到用户下一步动作。

## 5. 设计演进计划
- 先完成设计补齐与互链回写。
- 再按项目文档任务拆解推进实现与回归。
