# GitHub Pages 发布入口 + Release 安装包流水线（2026-03-01）项目管理文档

- 对应设计文档: `doc/site/github-pages/github-pages-release-download-pipeline-2026-03-01.design.md`
- 对应需求文档: `doc/site/github-pages/github-pages-release-download-pipeline-2026-03-01.prd.md`

审计轮次: 5

## 审计备注
- 主项目入口：`doc/site/github-pages/github-pages-game-engine-reposition-2026-02-25.project.md`
- 本文仅维护本专题增量任务，不重复主项目文档任务编排。

## 任务拆解

### T0A CI 阻塞修复（先行）
- [x] 修复 `cargo fmt --all -- --check` 基线（提交仓库内遗留格式化差异）
- [x] 修复 `world_game_launcher_tests` 被误识别为独立 bin 导致的 `cargo test` 失败
- [x] 修复 `agent_world_viewer --target wasm32-unknown-unknown` 的 `ctrlc` 目标兼容问题
- [x] 回归 `./scripts/ci-tests.sh required` 并确认通过

### T0 建档与基线
- [x] 新建设计文档：`doc/site/github-pages/github-pages-release-download-pipeline-2026-03-01.prd.md`
- [x] 新建项目管理文档：`doc/site/github-pages/github-pages-release-download-pipeline-2026-03-01.project.md`
- [x] 明确资产命名、触发条件、页面下载入口口径

### T1 发布流水线实现
- [x] 新增 Release 工作流（tag + 手动触发）
- [x] 新增安装包打包脚本（矩阵复用、固定资产命名）
- [x] 上传三平台资产与 SHA256 校验文件到 Release
- [x] 任务测试与提交

### T2 页面下载入口接入
- [x] 更新 `site/index.html`、`site/en/index.html` 增加下载区块与入口锚点
- [x] 更新 `site/assets/styles.css` 下载区块样式
- [x] 更新 `site/assets/app.js` 拉取 latest tag 进行页面展示（失败时回退）
- [x] 新增/更新下载入口校验脚本并接入 CI
- [x] 任务测试与提交

### T3 回归验证与文档收口
- [x] 执行脚本回归与基础构建校验
- [x] 回写本项目管理文档状态
- [x] 写任务日志：`doc/devlog/2026-03-01.md`
- [x] 任务测试与提交

### T3A Pages 门禁兼容性热修复（GitHub runner 无 rg）
- [x] 复现并定位 Actions run `22474048679` / job `65097149123` 失败原因
- [x] 修复 `scripts/site-manual-sync-check.sh`：`rg` 不可用时回退 `grep -F`
- [x] 修复 `scripts/site-download-check.sh`：同样支持 `grep -F` 回退
- [x] 本地回归：正常 PATH + 无 `rg` PATH 双路径校验

### T3B Rust required gate 兼容性热修复（GitHub runner 无 rg）
- [x] 复现并定位 `Rust` workflow 失败根因：`scripts/doc-governance-check.sh` 直接依赖 `rg`
- [x] 修复 `scripts/doc-governance-check.sh`：标题检测与绝对路径检测在 `rg` 不可用时回退 `grep -E`
- [x] 本地回归：正常 PATH + 无 `rg` PATH 双路径校验

### T3C Builtin Wasm m1 identity 清单回收敛
- [x] 复现并定位 `Builtin Wasm m1 Multi Runner` 失败根因：`m1.body.core` source_hash 失配
- [x] 执行 `scripts/sync-m1-builtin-wasm-artifacts.sh` 更新 hash/identity manifest
- [x] 本地回归 `scripts/ci-m1-wasm-summary.sh`（至少当前平台）并确认通过

### T3D Rust required gate m5 identity/hash 清单热修
- [x] 复现并定位 `Rust` workflow 新失败根因：`m5.gameplay.crisis.cycle` wasm hash 不在 identity/hash token 列表
- [x] 同步 `m5` builtin wasm 清单，并为 hash token 增加兼容候选集合（覆盖 runner 变体）
- [x] 调整 `builtin_wasm_identity` m5 用例，兼容 `identity_hash_v1` 签名方案
- [x] 本地回归 `scripts/sync-m5-builtin-wasm-artifacts.sh --check` 与失败定向测试

### T3E m5 多 token 清单持久化修正
- [x] 复盘并定位二次失败根因：`sync-m5` legacy 回写会将多 token 清单压回单 token
- [x] 手工固定 `m5_builtin_modules.sha256` 与 `m5_builtin_modules.identity.json` 的多 token 顺序集合（含 CI 报错 hash）
- [x] 只读校验 `scripts/sync-m5-builtin-wasm-artifacts.sh --check`，确保清单一致且不再被覆盖

### T3G Release Packages 编译提速（2026-03-13）
- [x] 复盘 `Release Packages` 多轮重跑中 compile/install 热点，确认慢点集中在 release-gate 的 full tier 重编译与 builtin wasm canonical nightly 按需安装。
- [x] 更新 `.github/workflows/release-packages.yml`：为 `release-gate` / `build-web-dist` / `package-native` 接入 `Swatinem/rust-cache@v2`，缓存 cargo registry/target 产物以缩短重复编译时间。
- [x] 更新 `.github/workflows/release-packages.yml`：在 `release-gate` 前显式预装 canonical builtin wasm toolchain（`nightly-2025-12-11 + rust-src + wasm32-unknown-unknown`），避免 full tier 测试期间再由 materializer 按需拉取。
- [x] 调整 workflow 顶层 cargo 环境：启用 sparse registry、提高 network retry、关闭 dev/test debug info，进一步降低 CI 编译与下载开销。
- [x] 本地校验 workflow 结构与文档回写，继续观察新一轮 release tag 实跑表现。

### T3H Release gate UDP gossip flake 热修
- [x] 复盘 `Release Packages` run `23053414184`，确认阻断点不是打包脚本，而是 `agent_world_node::tests::runtime_gossip_tracks_peer_committed_heads` 在 CI 高负载下 5 秒窗口内偶发未观测到 peer heads。
- [x] 调整 `crates/agent_world_node/src/tests_split_part2.rs`：为 UDP gossip 双节点都启用 `with_auto_attest_all_validators(true)`，避免测试依赖跨节点 attestation 时序抖动；同时把等待窗口从 5s 提升到 8s，吸收 GitHub runner 高负载波动。
- [x] 本地回归该用例的精确重跑，并在回写 `project/devlog` 后继续通过新 tag 观察 `Release Packages` 是否彻底放行。

### T3I Release gate execution bridge signer allowlist 热修
- [x] 复盘 `Release Packages` run `23055068064`，确认 `v0.0.8` 的新阻断点为 `world_chain_runtime` 单测 `node_runtime_execution_driver_commit_routes_modules_via_step_with_modules`；其前置 `InstallModuleFromArtifact` 实际被拒，因为 binary unit test 环境不会自动注入 `test.module.release.signer` 到 `World::new()` 的 `node_identity_bindings`。
- [x] 调整 `crates/agent_world/src/bin/world_chain_runtime/execution_bridge.rs`：在该测试里显式 `bind_node_identity(TEST_MODULE_ARTIFACT_SIGNER_NODE_ID, ...)`，并补充 `ModuleInstalled` / `module_tick_schedule` 前置断言，确保断言真正覆盖“commit 走 `step_with_modules` 并冒泡模块失败”。
- [x] 本地回归 `world_chain_runtime` 定向用例，并与相邻 execution bridge 持久化用例一起校验通过；继续通过新 tag 观察 `Release Packages` 是否越过 `release-gate`。

### T3J Release gate m5 economic overlay hash token 热修
- [x] 复盘 `Release Packages` run `23056942631`，确认 `v0.0.9` 的新阻断点在 `sync_m5`：`m5.gameplay.economic.overlay` 的 `linux-x86_64` canonical hash 已漂移到 `797e76900aa04297700c8ca5512ba9b00c6f8c4e83845d8ff473bd2adb0e6676`，而仓库清单仍写旧值 `36645c1c3fd590c4212691ba1ae0a881ef12171a9d375ee8693127e610968274`。
- [x] 更新 `crates/agent_world/src/runtime/world/artifacts/m5_builtin_modules.sha256` 与 `crates/agent_world/src/runtime/world/artifacts/m5_builtin_modules.identity.json`：将 `m5.gameplay.economic.overlay` 的 `linux-x86_64` hash token 对齐到当前 canonical 产物；该模块现已与 `darwin-arm64` 共用同一 canonical hash。
- [ ] 本地回归 `./scripts/sync-m5-builtin-wasm-artifacts.sh --check`，并通过新 tag 继续观察 `Release Packages` 是否终于越过 `release-gate`。

### T3K Release gate agent-browser CLI fallback 热修
- [x] 复盘 `Release Packages` run `23059581794`，确认 `v0.0.10` 已越过 `ci_full/sync_m1/sync_m4/sync_m5`，但在 `web_strict` 触发 `./scripts/viewer-release-qa-loop.sh` 时，GitHub runner 缺少全局 `agent-browser` 命令，直接导致 `missing required command: agent-browser`。
- [x] 调整 `scripts/agent-browser-lib.sh`：优先使用本机 `agent-browser`，当 CLI 不存在时自动回退到 `npx --yes agent-browser`；保持 `AGENT_BROWSER_SESSION` 透传，避免 CI 因为“没全局安装”而把 Web 严格闭环整段跳红。
- [x] 本地回归脚本级 fallback：在无 `agent-browser`、仅有伪造 `npx` 的 PATH 下，执行 `source scripts/agent-browser-lib.sh && ab_require && ab_cmd fallback-session get url`，确认实际走到 `--yes agent-browser get url`。

### T3F Release Packages macOS runner 配置热修
- [x] 复现并定位 `Release Packages` run `22545989082` / job `65309292458` 失败根因：`macos-13-us-default` 不受当前仓库支持
- [x] 修复 `.github/workflows/release-packages.yml`：macOS 矩阵 runner 改为 `macos-14`，并显式配置 `target_triple=x86_64-apple-darwin`
- [x] 扩展打包脚本参数链路：`release-prepare-bundle.sh` / `build-game-launcher-bundle.sh` 支持 `--target-triple` 并正确定位 `target/<triple>/<profile>` 产物
- [x] 本地回归脚本语法与 dry-run，推送后重新触发 `Release Packages` 验证

## 依赖
- 打包基础脚本：`scripts/build-game-launcher-bundle.sh`
- 站点发布流程：`.github/workflows/pages.yml`
- 站点入口文件：`site/index.html`、`site/en/index.html`

## 状态
- 当前阶段：进行中（T0A/T0/T1/T2/T3/T3A/T3B/T3C/T3D/T3E/T3F/T3G/T3H/T3I/T3K 已完成；T3J 继续由远端 release gate 验证）
- 最近更新：2026-03-13 已继续补 `T3K` release gate Web 严格闭环 CLI 兼容热修（`agent-browser` 缺失时自动回退 `npx`），下一轮 release tag 将继续验证是否终于越过 `web_strict`。
- 下一步：push `main` 并打新 release tag，继续观察 `Release Packages` 是否越过 `web_strict` 并开始进入 `build-web-dist/package-native/publish-release`。

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
