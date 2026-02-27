# 可发行统一启动器（Launcher）设计文档（2026-02-27）

## 目标
- 提供一个面向玩家的统一启动入口，不再要求玩家手动分别启动后端与 Web 前端。
- 启动器在一次命令内完成：启动 `world_viewer_live`、提供 Web Viewer 静态资源服务、拼接连接 URL 并自动打开浏览器。
- 发行口径下不依赖 `trunk serve` 作为运行时前置，改为消费预构建静态资源目录。

## 范围
- 新增 `world_game_launcher` 二进制（归属 `agent_world` crate）。
- 启动器负责：
  - 参数解析与默认值（bind/web-bind/viewer host/port/scenario/llm 开关）。
  - 托管 `world_viewer_live` 子进程生命周期。
  - 内置最小静态 HTTP 服务（读取预构建 web 目录）。
  - 输出玩家可访问 URL，可选自动打开浏览器。
- 新增发行打包脚本：构建 launcher + `world_viewer_live` + web dist 到统一目录。

## 非目标
- 不在本阶段引入 GUI 桌面启动器（Electron/Tauri）。
- 不改动 `world_viewer_live` 的核心业务逻辑与协议。
- 不处理自动更新/增量补丁系统。

## 接口/数据
### 启动器 CLI（初版）
- `--scenario <name>`：默认 `llm_bootstrap`。
- `--live-bind <host:port>`：默认 `127.0.0.1:5023`。
- `--web-bind <host:port>`：默认 `127.0.0.1:5011`。
- `--viewer-host <host>`：默认 `127.0.0.1`。
- `--viewer-port <port>`：默认 `4173`。
- `--viewer-static-dir <path>`：默认 `web`（发行包目录）；开发态可传入 `crates/agent_world_viewer/dist`。
- `--with-llm`：默认关闭（与现有可玩性测试口径一致）。
- `--no-open-browser`：默认自动打开浏览器。

### 运行时数据
- URL 组装：`http://{viewer-host}:{viewer-port}/?ws=ws://{web-bind-host}:{web-bind-port}`。
- 启动产物目录：launcher 仅输出日志到 stdout/stderr（后续可扩展日志落盘参数）。
- 打包目录（脚本输出）：
  - `bin/world_game_launcher`
  - `bin/world_viewer_live`
  - `web/`（trunk build 产物）

## 里程碑
- M1：Launcher CLI + 子进程托管 + URL 输出闭环可运行。
- M2：内置静态 HTTP 服务可稳定服务 trunk build 产物。
- M3：发行打包脚本完成并可一次性生成可分发目录。
- M4：文档更新，给出玩家启动命令与回归测试记录。

## 手册入口（落地命令）
- 构建发行包：
  - `./scripts/build-game-launcher-bundle.sh --out-dir output/release/game-launcher-local`
- 启动发行包（统一入口）：
  - `output/release/game-launcher-local/run-game.sh`
- 启动 launcher（工作区直跑）：
  - `env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_game_launcher -- --viewer-static-dir crates/agent_world_viewer/dist`

## 完成态（2026-02-27）
- `world_game_launcher` 已提供统一入口：后端 `world_viewer_live` + 内置静态 HTTP 服务 + URL 自动打开。
- 启动器具备基础路径安全（拒绝目录穿越）与 SPA 路由回退能力。
- `scripts/build-game-launcher-bundle.sh` 已能输出可分发目录（`bin/ + web/ + run-game.sh`）。

## 风险
- 静态服务器实现过于简化可能导致浏览器资源请求兼容问题。
  - 缓解：补基础 MIME/路径安全处理与单元测试。
- 发行包路径假设（可执行文件同级）在不同平台可能偏差。
  - 缓解：支持 `--viewer-static-dir` 显式覆盖，并在错误时输出可执行修复建议。
- LLM 默认开启会引入外部依赖与启动失败概率。
  - 缓解：首版默认 `--no-llm`，由用户显式开启。
