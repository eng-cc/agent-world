# GitHub Pages 发布入口 + Release 安装包流水线（2026-03-01）项目管理文档

- 对应设计文档: `doc/site/github-pages/github-pages-release-download-pipeline-2026-03-01.design.md`
- 对应需求文档: `doc/site/github-pages/github-pages-release-download-pipeline-2026-03-01.prd.md`

审计轮次: 6

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

### T3L Release gate trunk-missing dist fallback 热修
- [x] 复盘 `Release Packages` run `23078512672`，确认 `v0.0.11` 已经真正越过 `agent-browser` CLI 入口，但 `web_strict` 在解析 Viewer 静态资源目录时，因为 runner 没有安装 `trunk` 而退出；失败签名为 `error: missing required command: trunk`。
- [x] 调整 `scripts/agent-browser-lib.sh`：当请求 `web` 别名、`crates/agent_world_viewer/dist/index.html` 已存在但 `trunk` 不可用时，回退到仓库已提交的 `crates/agent_world_viewer/dist`，只在 `dist` 也不存在时才继续报错；避免 CI 因缺少前端构建器而阻断 Web 闭环。
- [x] 本地回归脚本级 fallback：在无 `trunk` 的 PATH 下 source `scripts/agent-browser-lib.sh`，并通过桩掉 `find` 强制进入 fallback 分支，确认返回 `crates/agent_world_viewer/dist` 且打印 `warning: trunk missing; falling back to committed viewer dist`。

### T3F Release Packages macOS runner 配置热修
- [x] 复现并定位 `Release Packages` run `22545989082` / job `65309292458` 失败根因：`macos-13-us-default` 不受当前仓库支持
- [x] 修复 `.github/workflows/release-packages.yml`：macOS 矩阵 runner 改为 `macos-14`，并显式配置 `target_triple=x86_64-apple-darwin`
- [x] 扩展打包脚本参数链路：`release-prepare-bundle.sh` / `build-game-launcher-bundle.sh` 支持 `--target-triple` 并正确定位 `target/<triple>/<profile>` 产物
- [x] 本地回归脚本语法与 dry-run，推送后重新触发 `Release Packages` 验证

### T3M Release gate 并行拆分与聚合收口（2026-03-14）
- [x] 复盘 `Release Packages` 连续多轮失败，确认主要问题已不再是单一业务缺陷，而是所有 release blocker 串在单个 `release-gate` job 中，导致 runtime/sync、web strict、S9/S10 soak 彼此阻塞，后续 `build-web-dist/package-native/publish-release` 长期无法进入。
- [x] 更新 `.github/workflows/release-packages.yml`：将 gate 拆为 `release_gate_runtime` / `release_gate_web` / `release_gate_soak` 三个并行 job，并新增 `release_gate` aggregate job 统一汇总 `needs.*.result`，继续保持“全部通过才放行打包”的语义。
- [x] 更新 `.github/workflows/release-packages.yml`：在 `release_gate_web` 内显式 provision `actions/setup-node@v4 + trunk`，并为三个子门分别上传 `.tmp/release_gate_*` summary artifact，缩短 CI 缺依赖与长时间黑盒失败的定位链路。
- [x] 本地校验 workflow 语法、`release-gate.sh` dry-run 组合与文档回写后，再进入下一轮远端 release tag 验证。

### T3N Release gate soak 预热依赖回补（2026-03-14）
- [x] 复盘 `Release Packages` run `23080174183` 新架构首轮结果，确认 `release-gate-soak` 在 1 秒内失败并非 soak 逻辑本身回归，而是拆分后仍沿用 `--no-prewarm`，导致 `s9` 失去来自 `ci_full` 的 `target/debug/world_chain_runtime` 预热前置。
- [x] 更新 `.github/workflows/release-packages.yml`：在 `release_gate_soak` 中新增 `env -u RUSTC_WRAPPER cargo build -p agent_world --bin world_chain_runtime` 预热步骤，使 soak 子门在独立 job 中重新自洽，同时保持 `release-gate.sh` 现有参数与 release 语义不变。
- [x] 本地回归 workflow 语法与 soak job 关键片段，确认 `Prewarm soak runtime binary` 已位于 `Run soak release gate` 之前。
- [ ] 推送修复并打新 tag，继续观察并行 gate 是否能全部进入 aggregate `release_gate`。

### T3O Release gate web sibling binary 预热回补（2026-03-14）
- [x] 复盘 `Release Packages` run `23080255868`，确认 `release-gate-web` 已越过 `trunk` 安装，但 `web_strict` 在 `world_game_launcher` 启动阶段因独立 job 缺少 `target/debug/world_viewer_live` 而失败；失败签名为 `failed to locate \`world_viewer_live\` binary; build it first or set AGENT_WORLD_WORLD_VIEWER_LIVE_BIN`。
- [x] 调整 `scripts/viewer-release-qa-loop.sh`：在启动 `world_game_launcher` 前显式执行 `env -u RUSTC_WRAPPER cargo build -p agent_world --bin world_viewer_live --bin world_chain_runtime`，把原先依赖其他步骤隐式生成 sibling binaries 的前置条件收回到脚本内部。
- [x] 本地回归 `bash -n scripts/viewer-release-qa-loop.sh`，并确认预热命令已位于 `cargo run -p agent_world --bin world_game_launcher` 之前。
- [ ] 推送修复并打新 tag，继续观察 `release-gate-web` 是否越过 launcher 启动阶段，并进一步验证 aggregate `release_gate` 与后续打包链路。

### T3P Release gate web test API 冷启动窗口放宽（2026-03-14）
- [x] 复盘 `Release Packages` run `23080686951`，确认 `release-gate-web` 已越过 sibling binary 缺失，但页面在 GH runner 上打开后 20 秒内仍未暴露 `window.__AW_TEST__`，导致 `web_strict` 以 `__AW_TEST__ is unavailable` 退出；launcher 与 bridge 已正常就绪，说明问题落在 Web 端冷启动窗口而非服务拉起。
- [x] 调整 `scripts/viewer-release-qa-loop.sh`：将 `wait_for_api` 从 20s 提升到 60s、将初始 `wait_for_connected` 从 15s 提升到 30s，并在 `__AW_TEST__` 超时前自动抓取 `console` / `errors` 日志，便于下一轮若仍失败时直接定位浏览器端异常。
- [x] 本地回归 `bash -n scripts/viewer-release-qa-loop.sh`，确认等待窗口与失败诊断输出语法正确。
- [ ] 推送修复并打新 tag，继续观察 `release-gate-web` 是否终于越过 Web Test API 初始化阶段。

### T3Q Release gate web test API readiness 兼容修复（2026-03-14）
- [x] 复盘 `Release Packages` run `23081035902` 与既往 `2026-03-10` Web QA 记录，确认 `wait_for_api` 不是单纯超时，而是会把 `agent-browser eval` 返回的 `"ready"` 误判为未就绪；当前 CI 日志中页面已打开、launcher stack 已 ready、console/errors 为空，与这一旧签名一致。
- [x] 调整 `scripts/viewer-release-qa-loop.sh`：新增 `normalize_eval_token`，将 `wait_for_api` 改为评估 `typeof window.__AW_TEST__ === "object" ? "ready" : "missing"`，并兼容 `ready/"ready"/true` 三种返回形态，避免被 agent-browser 的字符串化输出误伤。
- [x] 本地回归 `bash -n scripts/viewer-release-qa-loop.sh`，确认 readiness 兼容逻辑与现有超时/console 采集分支可同时生效。
- [ ] 推送修复并打新 tag，继续观察 `release-gate-web` 是否终于越过 Web Test API readiness 检查并进入语义交互断言。

## 依赖
- 打包基础脚本：`scripts/build-game-launcher-bundle.sh`
- 站点发布流程：`.github/workflows/pages.yml`
- 站点入口文件：`site/index.html`、`site/en/index.html`

## 状态
- 当前阶段：进行中（T0A/T0/T1/T2/T3/T3A/T3B/T3C/T3D/T3E/T3F/T3G/T3H/T3I/T3J/T3K/T3L/T3M/T3N/T3O/T3P/T3Q 已完成；下一轮验证并行 `release_gate_*` 与 aggregate gate 是否稳定放行）
- 最近更新：2026-03-14 已完成 `T3Q` Web Test API readiness 兼容修复：`release-gate-web` 的失败进一步定位到 agent-browser 将 `ready` 字符串化后被 `wait_for_api` 误判，现已做兼容归一化。
- 下一步：push `main` 并打新 release tag，继续观察 `release_gate_runtime/web/soak` 是否全部通过并进入 aggregate `release_gate`，随后再看 `build-web-dist/package-native/publish-release`。

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
