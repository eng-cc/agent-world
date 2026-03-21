# 客户端启动器 egui Web 同层复用与静态资源服务（2026-03-04）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.project.md`

审计轮次: 5


## 1. Executive Summary
- Problem Statement: 当前无 GUI 服务器场景的 `oasis7_web_launcher` 使用独立 HTML 控制台，未与 `oasis7_client_launcher` 的 egui UI 层复用，导致交互行为与文案演进仍存在双端分叉风险。
- Proposed Solution: 将 `oasis7_client_launcher` 改造为同一套 egui UI 代码跨 native/wasm 双目标运行；`oasis7_web_launcher` 改为托管 launcher wasm 静态资源并继续提供进程控制 API。
- Success Criteria:
  - SC-1: native 与 web 启动器使用同一份 egui UI 代码（同一 crate、同一字段映射、同一渲染结构）。
  - SC-2: `oasis7_web_launcher` 不再依赖内嵌 HTML，改为启动静态资源服务并提供 SPA 首页入口。
  - SC-3: launcher wasm 构建产物可纳入 bundle，headless 服务器开箱可访问。
  - SC-4: 关键启动/停止/状态日志闭环在 web 端通过 egui UI 可用且可回归。

## 2. User Experience & Functionality
- User Personas:
  - 启动器开发者：希望 UI 代码单点维护，避免 native/web 双套前端。
  - 运维人员：希望 headless 服务器上仍能获得与桌面一致的控制台体验。
- User Scenarios & Frequency:
  - 每次启动器 UI 变更后，需同时覆盖 native 与 web（高频，随功能迭代）。
  - 每次发布 bundle 时，需确保 `run-web-launcher.sh` 默认可访问 egui web 启动器（每次发布）。
- User Stories:
  - As a 启动器开发者, I want launcher UI to run on native/wasm from the same egui code, so that I only maintain one UI layer.
  - As an 运维人员, I want oasis7_web_launcher to serve launcher static assets directly, so that I can manage game startup remotely from browser on headless servers.
- Critical User Flows:
  1. Flow-LAUNCHER-EGUI-001（同层复用）:
     `修改 launcher egui UI 代码 -> native 启动器生效 -> wasm 启动器生效`
  2. Flow-LAUNCHER-EGUI-002（headless 访问）:
     `run-web-launcher.sh 启动 -> 浏览器访问 / -> 加载 wasm 静态资源 -> 调用 /api/state|start|stop`
  3. Flow-LAUNCHER-EGUI-003（打包发行）:
     `build-game-launcher-bundle.sh -> 生成 web 与 web-launcher 双静态目录 -> 远程启动并验证入口可达`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| launcher egui 双目标 | `LaunchConfig` + shared UI schema 字段映射 | 同一 egui 组件渲染配置区与启动控制 | `idle -> running/stopped/failed` | section 按 schema 顺序渲染 | 本地 UI 会话 / 浏览器会话 |
| web 静态资源托管 | `console_static_dir`（CLI/ENV 可配） | GET `/` 返回 index.html，GET 资源路径返回 wasm/js/css | `boot -> static_ready` | 静态资源路径需防目录穿越 | 默认部署于受信网络 |
| bundle 产物扩展 | `web/` + `web-launcher/` | 构建时分别产出 viewer 与 launcher wasm 资源 | `build -> bundle_ready` | 目录结构固定，脚本入口注入环境变量 | 构建者可写输出目录 |
- Acceptance Criteria:
  - AC-1: `oasis7_client_launcher` 可在 `wasm32-unknown-unknown` 目标下编译并运行 egui web 启动器入口。
  - AC-2: `oasis7_web_launcher` 支持配置并托管 launcher 静态资源目录，首页返回 launcher wasm 页面。
  - AC-3: `build-game-launcher-bundle.sh` 产物新增 `web-launcher/`，`run-web-launcher.sh` 默认指向该目录。
  - AC-4: `/api/state`、`/api/start`、`/api/stop` 在 egui web UI 上形成可操作闭环。
  - AC-5: 现有 native 启动器主流程保持可用，不引入回归。
- Non-Goals:
  - 不在本轮统一 web/native 的所有附加弹窗能力（如反馈、转账、设置中心深度编辑）。
  - 不在本轮改造 `oasis7_game_launcher` 与 `world_chain_runtime` 的底层进程协议。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属需求。

## 4. Technical Specifications
- Architecture Overview:
  - `oasis7_client_launcher` 采用 `cfg(target_arch)` 双入口：native `run_native` + wasm `WebRunner`。
  - native 与 wasm 共享 launcher UI 渲染逻辑与 schema 字段映射。
  - `oasis7_web_launcher` 扩展静态文件服务，托管 launcher wasm 产物并保留既有 API。
  - bundle 脚本新增 launcher wasm 构建阶段，输出 `web-launcher/`。
- Integration Points:
  - `crates/oasis7_client_launcher/src/main.rs`
  - `crates/oasis7_client_launcher/src/app_process.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher.rs`
  - `scripts/build-game-launcher-bundle.sh`
  - `doc/world-simulator/prd.md`
  - `doc/world-simulator/project.md`
- Edge Cases & Error Handling:
  - 静态资源目录不存在：`oasis7_web_launcher` 返回可诊断 404/错误提示，不影响 API 路径。
  - wasm 端 API 超时/HTTP 异常：UI 状态回落为 `query_failed/start_failed/stop_failed` 并写入日志。
  - 目录穿越请求：静态服务必须拒绝 `..` 路径，避免越权读取。
  - 服务器仅绑定内网地址：UI 使用相对路径访问 API，避免硬编码 host 导致跨网段失效。
- Non-Functional Requirements:
  - NFR-1: launcher wasm 页面首屏可交互时间（本地网络）`p95 <= 2s`。
  - NFR-2: `/api/state` 轮询模式下 1s 刷新不导致 UI 卡死或请求堆积。
  - NFR-3: Rust 单文件行数约束持续满足（`oasis7_web_launcher.rs`、`main.rs` 均 <=1200）。
  - NFR-4: 打包入口脚本在 release/dev 两种 profile 下均可生成 `web-launcher/`。
- Security & Privacy:
  - 静态服务仅暴露指定目录内文件，不暴露父目录内容。
  - API 与静态资源不包含密钥类敏感信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1: PRD 建模与任务拆解。
  - M2: launcher egui wasm 化与 oasis7_web_launcher 静态托管。
  - M3: bundle 脚本接入、回归测试与文档收口。
- Technical Risks:
  - 风险-1: `oasis7_client_launcher` 现有 native-only 模块在 wasm 构建下出现编译路径冲突。
  - 风险-2: 静态资源托管与 API 路由冲突导致 404 或错误转发。
  - 风险-3: 打包阶段新增 trunk 构建使耗时上升，需要控制失败诊断可读性。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-012 -> TASK-WORLD_SIMULATOR-027/028 -> `test_tier_required`。
- Decision Log:
  - DEC-LAUNCHER-EGUI-001: 采用“同一 launcher egui crate 双目标编译 + oasis7_web_launcher 静态托管”方案，而非继续维护独立 HTML 控制台。理由：可从代码结构层面消除 UI 分叉并降低后续维护成本。
