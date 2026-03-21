# 启动器 Web 控制台设计（2026-03-04）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.project.md`

## 1. 设计定位
定义面向无图形会话服务器的 `oasis7_web_launcher`：通过 HTTP 控制台页面与 API 托管 `oasis7_game_launcher` 子进程，提供远程启停、状态轮询与日志观察能力。

## 2. 设计结构
- 二进制托管层：`oasis7_web_launcher` 负责进程生命周期与日志缓存。
- 控制面 API 层：暴露 `/api/start`、`/api/stop`、`/api/state` 等接口。
- Web 页面层：提供浏览器可用的控制台表单与状态视图。
- 打包入口层：发行包输出 Web 控制台启动脚本，便于远程部署。

## 3. 关键接口 / 入口
- `GET /`
- `GET /api/state`
- `POST /api/start`
- `POST /api/stop`
- `oasis7_web_launcher.rs`
- `build-game-launcher-bundle.sh`

## 4. 约束与边界
- 默认部署前提仍是受信网络，不内建账号体系或 RBAC。
- 非法配置不得触发启动，需返回明确错误详情。
- 状态轮询要轻量稳定，日志缓存需有上限。
- 本阶段只做 launcher 过程控制，不替代 viewer 业务面板。

## 5. 设计演进计划
- 先落地无 GUI Web 控制台二进制与基础 API。
- 再补打包入口与远程部署支持。
- 最后用定向测试与文档回写确认 headless 场景闭环。
