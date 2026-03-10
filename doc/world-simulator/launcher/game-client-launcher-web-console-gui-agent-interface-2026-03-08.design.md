# 启动器 Web Console GUI Agent 全量接口设计（2026-03-08）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-web-console-gui-agent-interface-2026-03-08.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-console-gui-agent-interface-2026-03-08.project.md`

## 1. 设计定位
定义 `world_web_launcher` 面向 GUI Agent 的单一机器接口：通过 `/api/gui-agent/*` 提供能力发现、状态别名与统一动作执行，使自动化代理无需拼接分散的人工控制面端点。

## 2. 设计结构
- 能力发现层：`/api/gui-agent/capabilities` 暴露可用动作全集与参数约束。
- 状态快照层：`/api/gui-agent/state` 提供与 `/api/state` 对齐的机器可读状态别名。
- 动作执行层：`/api/gui-agent/action` 统一封装控制、提交与查询动作。
- 查询白名单层：通过固定 `query_target` 映射 explorer/transfer 查询，拒绝任意路径穿透。
- 统一响应层：所有动作返回 `{ ok, action, error_code?, error?, data?, state }`。

## 3. 关键接口 / 入口
- `GET /api/gui-agent/capabilities`
- `GET /api/gui-agent/state`
- `POST /api/gui-agent/action`
- `gui_agent_api.rs`
- `control_plane.rs`
- `transfer_query_proxy.rs`

## 4. 约束与边界
- 新接口必须复用现有控制面能力，不破坏 `/api/*` 旧路由语义。
- 查询目标采用白名单映射，不允许任意 URL 透传。
- 动作执行后总是返回最新 `state` 快照，减少 Agent 二次探测。
- 本阶段不引入鉴权/RBAC，仍以受信网络部署为前提。

## 5. 设计演进计划
- 先冻结 GUI Agent 动作枚举与统一响应结构。
- 再落地 `/api/gui-agent/*` 路由与内部复用桥接。
- 最后用定向测试验证人工控制面兼容与 Agent 全功能覆盖。
