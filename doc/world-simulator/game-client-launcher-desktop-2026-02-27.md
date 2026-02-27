# 可发行客户端启动器（Desktop）设计文档（2026-02-27）

## 目标
- 提供面向玩家的“客户端启动器”桌面应用，减少命令行操作门槛。
- 玩家可通过 GUI 完成：启动/停止游戏栈、查看当前连接地址、一键打开游戏页面。
- 与现有 `world_game_launcher` 复用同一运行链路，避免复制核心启动逻辑。

## 范围
- 新增桌面客户端启动器 crate：`crates/agent_world_client_launcher`。
- 启动器 GUI 提供：
  - 基础参数编辑（LLM 开关、viewer/live/web bind、静态资源目录）。
  - 启动/停止按钮与进程状态显示。
  - 一键打开游戏 URL。
  - 启动日志滚动展示（stdout/stderr 汇总）。
- 发行打包脚本纳入桌面启动器二进制，并生成 `run-client.sh` 入口。

## 非目标
- 本阶段不实现自动更新器。
- 不实现复杂账号系统、支付或在线补丁分发。
- 不改动 `world_viewer_live` 与 `world_game_launcher` 的核心业务协议。

## 接口/数据
### 桌面启动器行为
- 子进程：调用 `world_game_launcher`（并透传 GUI 参数）
- 默认参数：
  - scenario: `llm_bootstrap`
  - live bind: `127.0.0.1:5023`
  - web bind: `127.0.0.1:5011`
  - viewer host: `127.0.0.1`
  - viewer port: `4173`
  - viewer static dir: `web`
  - LLM: 默认启用
- URL：`http://{viewer-host}:{viewer-port}/?ws=ws://{web-bind-host}:{web-bind-port}`

### 打包目录
- `bin/agent_world_client_launcher`
- `bin/world_game_launcher`
- `bin/world_viewer_live`
- `web/`
- `run-client.sh`
- `run-game.sh`

## 里程碑
- M1：Desktop GUI 启动器 MVP（启动/停止/URL/日志）可运行。
- M2：打包脚本接入客户端启动器并可产出分发目录。
- M3：手册与项目文档收口，给出玩家可执行路径。

## 风险
- GUI 依赖在不同 Linux 环境下可能出现图形后端兼容差异。
  - 缓解：先以 `cargo check` 和单元测试保证构建链路，运行时提供 CLI fallback（`world_game_launcher`）。
- 子进程崩溃时 UI 状态不同步。
  - 缓解：轮询 `try_wait`，崩溃后自动切回未运行状态并写入日志。

## 手册入口（落地命令）
- 直跑桌面客户端启动器（开发态）：`env -u RUSTC_WRAPPER cargo run -p agent_world_client_launcher`
- 构建可分发目录：`./scripts/build-game-launcher-bundle.sh --out-dir output/release/game-launcher-local --web-dist site`
- 玩家入口（桌面 GUI）：`output/release/game-launcher-local/run-client.sh`
- CLI 兜底入口：`output/release/game-launcher-local/run-game.sh`

## 完成态（2026-02-27）
- M1 完成：`agent_world_client_launcher` 已具备参数编辑、启动/停止、日志显示与一键打开 URL。
- M2 完成：打包脚本已纳入 `agent_world_client_launcher` 并生成 `run-client.sh`。
- M3 完成：已补齐手册入口与验收记录，形成“桌面 GUI + CLI fallback”的可分发启动路径。
- 补充修复：客户端启动器接入 CJK 字体 fallback，默认内置中文字体，支持 `AGENT_WORLD_CLIENT_LAUNCHER_FONT` 覆盖字体路径，修复中文乱码问题。
- 补充修复：`build-game-launcher-bundle.sh` 的 release 构建改为分步构建，避免多包 `--bin` 过滤导致 `agent_world_client_launcher` 未产出。
