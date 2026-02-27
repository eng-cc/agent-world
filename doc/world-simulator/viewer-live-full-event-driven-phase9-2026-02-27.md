# Viewer Live 完全事件驱动改造 Phase 9（2026-02-27）

## 目标
- 彻底移除 `world_viewer_live` 与外围脚本中的旧 `--tick-ms` 入口，只保留 event-driven live 链路。
- 清理 viewer live 路径对“tick 驱动”参数的传递和使用，避免空跑配置继续暴露。
- 保持 node/runtime 共识 tick 机制不变（不在本阶段改造范围内）。

## 范围
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

## 接口/数据
- 删除 CLI 参数：`world_viewer_live --tick-ms`。
- 删除 `CliOptions.tick_ms` 字段，reward runtime 轮询改为复用 `node_tick_ms`。
- 删除脚本对 `--tick-ms` 的参数定义、校验与透传。
- 文档示例命令改为不含 `--tick-ms`。

## 里程碑
1. M0：建档（设计文档 + 项目管理文档）。
2. M1：CLI 与测试收敛，移除 live `tick-ms` 入口。
3. M2：脚本参数链路清理，统一仅走 event-driven live 启动方式。
4. M3：手册更新与 required 回归验证。

## 风险
- 现有自动化脚本/外部调用仍传 `--tick-ms` 可能直接失败，需要同步更新脚本与手册。
- reward runtime poll 改为复用 `node_tick_ms` 后，若用户仅调旧参数将失效，需通过 CLI 错误与文档清晰提示。
- 若误清理 node runtime tick 相关代码会引入共识回归，需要严格限定改造边界。

## Phase 9 完成态（T4）

### 交付结果
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

### 验收证据
- 代码回归（test_tier_required / full）：
  - `env -u RUSTC_WRAPPER cargo fmt --all -- --check`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world viewer::live::tests:: -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world world_viewer_live_tests:: -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features "viewer_live_integration test_tier_full" --test viewer_live_integration -- --nocapture`
- 文档残留扫描：
  - 活跃手册范围内 `--tick-ms` 已清零（仅历史 devlog/历史文档保留存档记录）。

### 阶段结论
- Phase 9 达成：viewer live 运行链路已去除旧 `tick-ms` 驱动入口，保留 event-driven live 语义；node/runtime 基础共识 tick 机制保持不变。
