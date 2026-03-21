# 客户端启动器 Web 控制台（2026-03-04）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.project.md`

审计轮次: 6

## 1. Executive Summary
- Problem Statement: 当前客户端启动器仅提供桌面 GUI，无法在无图形界面的服务器上直接运行并远程操作。
- Proposed Solution: 新增 `oasis7_web_launcher` Web 控制台二进制，通过浏览器提供启动/停止、状态查看、日志追踪与配置编辑能力。
- Success Criteria:
  - SC-1: 服务器仅通过命令行启动 `oasis7_web_launcher` 后，可在浏览器完成启动/停止主链路操作。
  - SC-2: Web 控制台可展示运行状态（`idle/running/stopped/invalid_config/start_failed/stop_failed/exited`）与最近日志，不依赖图形桌面会话。
  - SC-3: 启动参数校验失败时，Web 控制台返回结构化错误并阻止拉起子进程。
  - SC-4: 打包产物新增 Web 控制台入口脚本，发行包内可直接运行。

## 2. User Experience & Functionality
- User Personas:
  - 运维/测试人员：在远程 Linux 服务器上通过浏览器控制启动链路。
  - 发布人员：在无 GUI 环境验证 launcher 可用性并采集日志。
- User Scenarios & Frequency:
  - 每次远程环境部署后至少 1 次通过 Web 控制台验证启动链路。
  - 发布回归阶段按 `test_tier_required` 至少执行 1 次远程启动/停止闭环。
- User Stories:
  - As an 运维人员, I want to control launcher from browser, so that I can operate on headless servers without desktop UI.
  - As a 发布人员, I want to inspect launcher logs in web console, so that I can diagnose startup failures remotely.
- Critical User Flows:
  1. Flow-LAUNCHER-WEB-001（远程启动）:
     `SSH 启动 oasis7_web_launcher -> 浏览器访问控制台 -> 填写/确认配置 -> 点击启动 -> 观察 running 状态与游戏 URL`
  2. Flow-LAUNCHER-WEB-002（远程停止）:
     `控制台点击停止 -> 服务发送中断信号 -> 子进程退出 -> 状态回到 stopped`
  3. Flow-LAUNCHER-WEB-003（参数校验失败）:
     `提交非法 bind/端口/静态目录 -> API 返回 invalid_config -> 页面展示错误详情`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Web 启动器控制台 | `scenario/live_bind/web_bind/viewer_host/viewer_port/viewer_static_dir/llm/chain` | 点击“启动”提交配置并拉起 `oasis7_game_launcher` | `idle/stopped/exited -> running`，失败进入 `invalid_config/start_failed` | `viewer_port`、`bind` 必须可解析；静态目录必须存在 | 默认无鉴权，仅限受信网络部署 |
| 远程停止 | 无新增字段 | 点击“停止”优先优雅中断，超时后强杀 | `running -> stopped`，失败进入 `stop_failed` | 沿用优雅退出超时窗口与轮询策略 | 仅控制台会话可触发 |
| 状态与日志查询 | `status/pid/last_error/logs[]` | 页面轮询 `/api/state` 并刷新 UI | `polling` 常驻 | 仅保留最近 N 条日志（防止内存无限增长） | 只读 |
- Acceptance Criteria:
  - AC-1: `oasis7_web_launcher` 提供 `GET /` 控制台页面和 `/api/state` 状态接口。
  - AC-2: `POST /api/start` 可拉起 `oasis7_game_launcher`，并透传链路参数。
  - AC-3: `POST /api/stop` 可停止已运行子进程，状态可被轮询接口观测。
  - AC-4: 非法配置不会启动进程，接口返回明确错误信息。
  - AC-5: 打包脚本输出新增 Web 控制台入口脚本，支持发行包直接运行。
- Non-Goals:
  - 不在本轮实现账号体系、多租户鉴权或细粒度 RBAC。
  - 不替代 viewer 内业务面板，仅提供 launcher 过程控制。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- Architecture Overview:
  - 新增二进制：`crates/oasis7/src/bin/oasis7_web_launcher.rs`。
  - 运行时模型：`oasis7_web_launcher` 托管 `oasis7_game_launcher` 子进程，并通过内置 HTTP 服务暴露 Web 控制台与 API。
- Integration Points:
  - `crates/oasis7/src/bin/oasis7_game_launcher.rs`
  - `scripts/build-game-launcher-bundle.sh`
- Edge Cases & Error Handling:
  - 子进程已运行重复启动：返回冲突错误并保持现状。
  - 子进程非预期退出：状态标记为 `exited` 并记录退出码。
  - `viewer_static_dir` 不存在：返回配置错误，不触发启动。
  - bind/端口非法：返回参数错误详情。
  - HTTP 非法方法：返回 `405 Method Not Allowed`。
- Non-Functional Requirements:
  - NFR-1: `/api/state` 常态响应 `p95 <= 200ms`（本地网络）。
  - NFR-2: 日志缓存上限固定（默认 2000 条），避免内存线性增长。
  - NFR-3: 默认监听地址可配置为 `0.0.0.0`，满足远程访问场景。
- Security & Privacy:
  - 默认仅建议部署在受控内网或经反向代理保护的环境。
  - 日志与响应中不得输出密钥类敏感信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1: PRD 建模与任务拆解。
  - M2: `oasis7_web_launcher` 与 Web API 落地。
  - M3: 打包入口与文档收口。
- Technical Risks:
  - 风险-1: 无鉴权控制台暴露到公网存在运维风险。
  - 风险-2: 多次快速点击启动/停止可能触发状态竞争。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-010 -> TASK-WORLD_SIMULATOR-023/024 -> `test_tier_required`。
- Decision Log:
  - DEC-LAUNCHER-WEB-001: 采用“新增无 GUI Web 控制台二进制”而非扩展桌面 GUI 到远程渲染。理由：无界面服务器场景无需图形依赖，部署与运维成本更低。
