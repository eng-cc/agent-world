# 客户端启动器设置中心（含 LLM）（2026-03-02）

审计轮次: 3

- 对应项目管理文档: doc/world-simulator/launcher/game-client-launcher-llm-settings-panel-2026-03-02.prd.project.md

## 1. Executive Summary
- 在客户端启动器中提供“完整设置中心”，统一承载游戏、区块链与 LLM 配置入口。
- 在设置中心中可编辑并保存以下 LLM 核心字段：
  - `llm.api_key`
  - `llm.base_url`
  - `llm.model`
- 统一使用小写 TOML 风格，避免回退到 `AGENT_WORLD_LLM_*` 这类文件内非 TOML 风格写法。

## 2. User Experience & Functionality
### In Scope
- `crates/agent_world_client_launcher/src/main.rs`
  - 增加设置入口按钮与设置中心窗口 UI。
  - 设置中心覆盖游戏与区块链核心配置（场景、bind、二进制路径、链参数等）。
  - 增加 LLM 设置状态、保存结果提示。
  - 增加 `config.toml` 的 LLM 字段读取/写回逻辑。
- `crates/agent_world_client_launcher` 单元测试补充：
  - 读取已有 `[llm]` 字段。
  - 写入/更新/清空 `[llm]` 对应键。
  - 非法 TOML 输入时给出错误。

### Out of Scope
- 不改造 `world_game_launcher` 参数协议。
- 不新增 profile/provider 的可视化管理（仅覆盖本次明确要求的 3 个字段）。
- 不移除环境变量回退能力。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- 新增启动器 UI 入口：
  - 按钮：`设置 / Settings`
  - 交互：点击后弹出设置窗口（可关闭）。
- 设置窗口字段与 TOML 映射：
  - `API Key` -> `[llm].api_key`
  - `Base URL` -> `[llm].base_url`
  - `Model` -> `[llm].model`
- 读写策略：
  - 读取：从 `config.toml` 解析 `[llm]`，缺失字段按空字符串处理。
  - 保存：仅更新 `[llm]` 三个键；空字符串表示删除对应键；其余配置保持不变。
  - 若 `config.toml` 不存在，则创建最小 TOML 并写入 `[llm]`。

## 5. Risks & Roadmap
- M1（T0）：完成设计文档与项目管理文档。
- M2（T1）：完成设置按钮、设置窗口、读写逻辑与单测。
- M3（T2）：完成回归验证、文档完成态与 devlog 收口。

### Technical Risks
- 风险：直接写回 TOML 可能破坏用户手工注释或排序。
  - 缓解：限定写回范围为 `[llm]` 三个键，并在文档中说明保存行为。
- 风险：`config.toml` 内容非法导致读取失败。
  - 缓解：在 UI 中展示保存/读取错误并保留内存编辑值。
- 风险：用户清空字段后以为值仍生效。
  - 缓解：空字符串保存时删除键，提示“已清空并移除配置键”。

## 完成态（2026-03-02）
- 启动器操作区已新增“设置 / Settings”按钮，点击后可打开 LLM 设置窗口。
- 设置窗口已支持配置并保存以下字段到 `config.toml` 小写 TOML 键：
  - `llm.api_key`
  - `llm.base_url`
  - `llm.model`
- 配置读写逻辑已支持：
  - 启动/打开设置时从文件读取当前值；
  - 保存时仅更新 `[llm]` 三个键；
  - 空字符串保存时移除对应键；
  - 其余 TOML 表（如 `[node]`）保持不变。
- 单元测试与编译回归通过：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher`

## 增量完成态（2026-03-03）
- “设置 / Settings”窗口已升级为完整设置中心：
  - 游戏与显示配置：`scenario/live_bind/web_bind/viewer_host/viewer_port/viewer_static_dir/launcher_bin` 与 `llm_enabled/auto_open_browser`。
  - 区块链运行时配置：`chain_enabled/chain_runtime_bin/chain_status_bind/chain_node_id/chain_world_id/chain_node_role/chain_node_tick_ms/chain_node_validators`。
  - LLM 连接配置：保留 `llm.api_key/base_url/model` 文件重载与保存能力。
- 设置中心内游戏/区块链配置改动即时作用于启动器内存配置（无需额外保存按钮）；LLM 配置保持显式“保存到 config.toml / 从文件重载”流程。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
