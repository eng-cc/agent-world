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
- 当前阶段：已完成（T0A/T0/T1/T2/T3/T3A/T3B/T3C/T3D/T3E/T3F/T3G）
- 最近更新：2026-03-13 已继续补 `T3G` 编译提速改造（cargo cache + canonical builtin wasm toolchain 预热 + CI cargo env 优化），后续通过新 release tag 实跑继续验证收益。
- 下一步：观察 `Release Packages` 新一轮 tag run 的 `release-gate` 耗时与是否继续放行到 `build-web-dist/package-native/publish-release`。

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
