# Viewer Live 完全事件驱动改造 Phase 10（2026-02-27）

审计轮次: 2

## ROUND-002 物理合并
- 本文件为主文档（当前权威入口）。
- `phase8/phase9` 内容已物理合并入本文件，对应阶段文档降级为历史追溯。
- 历史阶段文档:
  - `doc/world-simulator/viewer/viewer-live-full-event-driven-phase8-2026-02-27.prd.md`
  - `doc/world-simulator/viewer/viewer-live-full-event-driven-phase9-2026-02-27.prd.md`
- 对应项目管理文档: doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.project.md

## 1. Executive Summary
- 清理 `agent_world::viewer` 活跃运行链路中残留的 tick 轮询逻辑，统一为事件驱动推进。
- 删除离线 viewer server 的定时回放推进（`tick_interval`），避免播放过程空 tick 空跑。
- 删除 web bridge 的可配置轮询间隔（`poll_interval`）及其轮询 sleep 链路，收敛为事件触发转发模型。

## 2. User Experience & Functionality
- `crates/agent_world/src/viewer/server.rs`
- `crates/agent_world/src/viewer/web_bridge.rs`
- `crates/agent_world/tests/viewer_offline_integration.rs`（如需同步）
- 与以上接口变更相关的调用点与测试
- 活跃手册/入口示例中的 viewer 旧 tick 参数残留（仅活跃文档，不改历史 devlog）

不在范围内：
- `agent_world_node` 共识/执行主循环中的 `tick_interval`（基础 runtime 机制，需单独阶段重构）
- 历史归档文档与历史 devlog

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- `ViewerServerConfig`：移除 `tick_interval` 字段及 `with_tick_interval`。
- `ViewerServer` 回放控制：`Play` 改为事件触发推进（一次请求驱动连续事件发出），不再依赖定时 tick。
- `ViewerWebBridgeConfig`：移除 `poll_interval` 字段及 `with_poll_interval`，清理轮询 sleep。

## 5. Risks & Roadmap
1. M0：建档（设计文档+项目管理文档）。
2. M1：offline server 去 tick 化并通过 required 测试。
3. M2：web bridge 去轮询化并通过 required 测试。
4. M3：文档与示例收口、阶段结项。

### Technical Risks
- 离线 `Play` 从“定速推送”变为“事件驱动批量推送”后，前端若依赖慢速动画可能表现变化。
- web bridge 去轮询后需确保断连退出、上游重连行为不退化。
- 若误触 node/runtime 基础 tick 机制，可能引入共识回归，需要严格边界控制。

## Phase 10 完成态（T4）

### 交付结果
- `viewer/server` 已去除定时回放推进链路：
  - 删除 `ViewerServerConfig.tick_interval` 与 `with_tick_interval`。
  - 主循环从 `recv_timeout + tick` 收敛为 `recv()` 请求驱动。
  - `Control::Play` 改为单次请求触发的连续事件输出，不再依赖定时 tick。
- `viewer/web_bridge` 已去除可配置轮询链路：
  - 删除 `ViewerWebBridgeConfig.poll_interval` 与 `with_poll_interval`。
  - 删除 `thread::sleep` 轮询，改为 socket 超时读 + 事件转发。
- 活跃入口与文档已清理旧 `--tick-ms` 示例：
  - `site/index.html`、`site/en/index.html`
  - `doc/world-simulator/viewer/visualization.prd.md`
  - `doc/world-simulator/viewer/viewer-i18n.prd.md`
  - `doc/world-simulator/viewer/viewer-open-world-sandbox-readiness.prd.md`
  - `doc/world-simulator/viewer/viewer-open-world-sandbox-readiness.stress-report.template.md`
  - `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.prd.md`
- 删除 legacy `--tick-ms` 拒绝断言测试，避免保留旧参数语义噪音：
  - `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests_split_part1.rs`

### 验收证据
- `env -u RUSTC_WRAPPER cargo fmt --all -- --check`
- `env -u RUSTC_WRAPPER cargo check -p agent_world`
- `env -u RUSTC_WRAPPER cargo test -p agent_world viewer::live::tests:: -- --nocapture`
- `env -u RUSTC_WRAPPER cargo test -p agent_world viewer::web_bridge::tests:: -- --nocapture`
- `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required --test viewer_offline_integration -- --nocapture`
- `rg --line-number --no-heading --glob '!doc/devlog/**' --glob '!doc/world-simulator/viewer/viewer-live-full-event-driven-phase9-2026-02-27*' -- \"--tick-ms\" .`

### 阶段结论
- Phase 10 达成：`agent_world::viewer` 活跃链路已移除旧 tick/poll 驱动入口与可配置轮询参数，viewer 运行路径收敛为事件驱动语义。
- 当前剩余 tick 仅在 node/runtime 基础机制及其测试中（例如 `agent_world_node` 与相关配置），不属于 viewer old-code 清理范围。

## Phase 8/9 增量记录（ROUND-002 物理合并）

### Phase 8：script 路径收敛为事件驱动
- 原阶段文档: `doc/world-simulator/viewer/viewer-live-full-event-driven-phase8-2026-02-27.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-full-event-driven-phase8-2026-02-27.prd.project.md`

#### 1. Executive Summary
- 将 script 路径默认且唯一节拍收敛为 `event_drive`，不再保留 `timer_pulse` 回退模式。
- 清理 script 回退链路代码（配置开关、分支判断、定时脉冲信号与线程）。
- 保持对外 viewer 协议不变，在不引入空跑 tick 的前提下维持 play/step/seek 行为。

#### 2. User Experience & Functionality
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `crates/agent_world/src/viewer/mod.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`

不在范围内：
- 不改动 viewer 对外协议字段。
- 不改动 node/runtime 共识协议与 reward runtime 机制。

#### 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

#### 4. Technical Specifications
- 删除 script 节拍策略开关：
  - 移除 `ViewerLiveScriptPacingMode`。
  - 移除 `ViewerLiveServerConfig.script_pacing_mode` 与 `with_script_pacing_mode`。
- script 非共识推进统一走 `NonConsensusDriveRequested` 事件链路。
- 清理 playback pulse 相关内部信号与统计项，保留事件驱动信号统计。

#### 5. Risks & Roadmap
1. M0：建档（设计文档 + 项目管理文档）。
2. M1：代码收敛到 script-only event-drive（删除回退开关与脉冲链路）。
3. M2：测试改造与 required 回归通过。
4. M3：文档收口与阶段结项。

##### Technical Risks
- 移除定时脉冲后，若 `Play` 初始触发链路遗漏，可能表现为不推进。
- 统计项调整后，已有日志解析脚本若依赖旧字段可能失配。
- 若仍有外部调用依赖已删除配置项，可能触发编译错误，需要同步收敛调用侧。

#### Phase 8 完成态（T3）

##### 交付结果
- script 路径默认且唯一推进模式已收敛为 `event_drive`：
  - 删除 `ViewerLiveScriptPacingMode` 与 `ViewerLiveServerConfig.script_pacing_mode`。
  - 删除 `with_script_pacing_mode` 配置入口。
- `timer_pulse` 回退链路已清理：
  - 删除 playback pulse 信号、控制结构、线程发射逻辑及对应主循环分支。
  - live backpressure 统计移除 playback pulse merge/drop 字段。
- 调用侧已同步：
  - `viewer/mod.rs` 不再导出 `ViewerLiveScriptPacingMode`。
  - `world_viewer_live` 不再向 live server 传递 tick-based pacing 配置。

##### 验收证据
- 回归测试（test_tier_required）：
  - `env -u RUSTC_WRAPPER cargo fmt --all -- --check`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world viewer::live::tests:: -- --nocapture`
- 仓库 pre-commit required 矩阵通过（`agent_world` / `agent_world_consensus` / `agent_world_distfs` / `agent_world_viewer`）。

##### 阶段结论
- Phase 8 达成：script 路径已切换为默认且唯一事件驱动，`timer_pulse` 回退开关与回退链路代码已清理。

#### 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。

### Phase 9：移除 live tick 入口与脚本透传
- 原阶段文档: `doc/world-simulator/viewer/viewer-live-full-event-driven-phase9-2026-02-27.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-full-event-driven-phase9-2026-02-27.prd.project.md`

#### 1. Executive Summary
- 彻底移除 `world_viewer_live` 与外围脚本中的旧 `--tick-ms` 入口，只保留 event-driven live 链路。
- 清理 viewer live 路径对“tick 驱动”参数的传递和使用，避免空跑配置继续暴露。
- 保持 node/runtime 共识 tick 机制不变（不在本阶段改造范围内）。

#### 2. User Experience & Functionality
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part2.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests_split_part1.rs`
- `crates/agent_world/tests/viewer_live_integration.rs`
- `scripts/capture-viewer-frame.sh`
- `scripts/viewer-theme-pack-preview.sh`
- `scripts/run-game-test.sh`
- `scripts/p2p-longrun-soak.sh`
- `scripts/viewer-owr4-stress.sh`
- `scripts/viewer-release-qa-loop.sh`
- `scripts/viewer-texture-inspector.sh`
- 活跃手册文档（testing/manual 与 viewer/manual 相关）

不在范围内：
- `agent_world_node` runtime 的 `tick_interval`（共识与执行调度基础机制）。
- 历史归档/历史 devlog 中的旧命令记录。

#### 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

#### 4. Technical Specifications
- 删除 CLI 参数：`world_viewer_live --tick-ms`。
- 删除 `CliOptions.tick_ms` 字段，reward runtime 轮询改为复用 `node_tick_ms`。
- 删除脚本对 `--tick-ms` 的参数定义、校验与透传。
- 文档示例命令改为不含 `--tick-ms`。

#### 5. Risks & Roadmap
1. M0：建档（设计文档 + 项目管理文档）。
2. M1：CLI 与测试收敛，移除 live `tick-ms` 入口。
3. M2：脚本参数链路清理，统一仅走 event-driven live 启动方式。
4. M3：手册更新与 required 回归验证。

##### Technical Risks
- 现有自动化脚本/外部调用仍传 `--tick-ms` 可能直接失败，需要同步更新脚本与手册。
- reward runtime poll 改为复用 `node_tick_ms` 后，若用户仅调旧参数将失效，需通过 CLI 错误与文档清晰提示。
- 若误清理 node runtime tick 相关代码会引入共识回归，需要严格限定改造边界。

#### Phase 9 完成态（T4）

##### 交付结果
- `world_viewer_live` 已删除 live 旧 tick 入口：
  - 移除 `--tick-ms` CLI 参数与 `CliOptions.tick_ms`。
  - reward runtime 轮询改为复用 `--node-tick-ms`。
- viewer live 外围脚本已全部移除 `--tick-ms` 参数链路：
  - `capture-viewer-frame` / `viewer-theme-pack-preview` / `run-game-test` /
    `p2p-longrun-soak` / `viewer-owr4-stress` / `viewer-release-qa-loop` /
    `viewer-texture-inspector`。
- 活跃手册和静态站 viewer manual 示例已同步更新为无 `--tick-ms` 版本。
- 修复事件驱动链路下的 live server 退出阻塞：
  - 主循环改为 `recv_timeout + loop_running`，客户端断开后可退出。

##### 验收证据
- 代码回归（test_tier_required / full）：
  - `env -u RUSTC_WRAPPER cargo fmt --all -- --check`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world viewer::live::tests:: -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world world_viewer_live_tests:: -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features "viewer_live_integration test_tier_full" --test viewer_live_integration -- --nocapture`
- 文档残留扫描：
  - 活跃手册范围内 `--tick-ms` 已清零（仅历史 devlog/历史文档保留存档记录）。

##### 阶段结论
- Phase 9 达成：viewer live 运行链路已去除旧 `tick-ms` 驱动入口，保留 event-driven live 语义；node/runtime 基础共识 tick 机制保持不变。

#### 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
