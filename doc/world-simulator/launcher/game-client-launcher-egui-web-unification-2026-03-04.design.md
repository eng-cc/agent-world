# 启动器 egui Web 同层复用与静态资源服务设计

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.project.md`

## 1. 设计定位
定义 launcher egui UI 在 native/wasm 双目标下的共享渲染结构，以及 `world_web_launcher` 的静态资源托管方式。

## 2. 设计结构
- UI 层：native/wasm 共享同一套 egui schema 与组件树。
- 服务层：`world_web_launcher` 负责托管 launcher wasm 静态资源并保留 API。
- 打包层：bundle 产出 `web-launcher/` 并作为 headless 默认入口。

## 3. 关键接口 / 入口
- `oasis7_client_launcher` native/wasm 双入口
- `world_web_launcher` 静态资源目录配置与 `/api/*` 控制面
- `build-game-launcher-bundle.sh` / `run-web-launcher.sh`

## 4. 约束与边界
- native 与 web 的字段映射、文案与交互顺序必须一致。
- 静态服务必须拒绝目录穿越并返回可诊断错误。
- 未配置 wasm 静态目录时不得影响 API 路径。

## 5. 设计演进计划
- 先完成设计补齐与互链回写。
- 再按项目文档任务拆解推进实现与回归。
