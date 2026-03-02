# 启动器 Viewer 鉴权自动继承 Node 配置（2026-03-02）

## 目标
- 通过启动器链路（`world_game_launcher` / `agent_world_client_launcher`）打开 Web Viewer 时，聊天与 Prompt 控制鉴权默认可用，不再要求用户手工设置 `AGENT_WORLD_VIEWER_AUTH_PUBLIC_KEY` 与 `AGENT_WORLD_VIEWER_AUTH_PRIVATE_KEY`。
- 将 Viewer 鉴权默认口径收敛到 `config.toml` 的 `[node] private_key/public_key`，让用户只需维护一处 keypair。
- 保持现有环境变量覆盖能力，兼容已有自动化与调试脚本。

## 范围
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world/src/bin/world_game_launcher/world_game_launcher_tests.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat_auth.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat_tests.rs`
- `crates/agent_world_viewer/Cargo.toml`

## 接口 / 数据
- 鉴权源数据：
  - `config.toml`
  - `[node] private_key`
  - `[node] public_key`
- Web 启动器注入：
  - `world_game_launcher` 在返回 `index.html` 时注入 `window.__AGENT_WORLD_VIEWER_AUTH_ENV`，键名与 Viewer 环境变量保持一致：
    - `AGENT_WORLD_VIEWER_PLAYER_ID`
    - `AGENT_WORLD_VIEWER_AUTH_PUBLIC_KEY`
    - `AGENT_WORLD_VIEWER_AUTH_PRIVATE_KEY`
- Viewer 解析优先级：
  - wasm（Web）：
    1. `window.__AGENT_WORLD_VIEWER_AUTH_ENV`
    2. 进程环境变量（兼容保留）
  - native：
    1. 环境变量（现状兼容）
    2. `config.toml` 的 `[node]` keypair 回退
- `player_id`：
  - 默认 `viewer-player`
  - 若存在 `AGENT_WORLD_VIEWER_PLAYER_ID` 则优先使用该值

## 里程碑
- M1：建档（设计 + 项目管理）。
- M2：`world_game_launcher` 注入 Web 鉴权配置并通过定向测试。
- M3：`agent_world_viewer` 增加 wasm 注入读取与 native `config.toml` 回退并通过定向测试。
- M4：文档/项目状态/devlog 收口。

## 风险
- Web 页面可见风险：若将 launcher 暴露到非本机网络，注入脚本会把私钥暴露给访问者。
  - 缓解：默认部署口径保持本机调试/单机发布；后续可补 loopback-only 强约束或显式开关。
- 配置一致性风险：`config.toml` 缺失或损坏时仍会回退失败。
  - 缓解：保留环境变量覆盖路径，报错信息明确包含回退失败原因。

