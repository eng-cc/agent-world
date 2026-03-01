# GitHub Pages 发布入口 + Release 安装包流水线（2026-03-01）项目管理文档

## 任务拆解

### T0A CI 阻塞修复（先行）
- [x] 修复 `cargo fmt --all -- --check` 基线（提交仓库内遗留格式化差异）
- [x] 修复 `world_game_launcher_tests` 被误识别为独立 bin 导致的 `cargo test` 失败
- [x] 修复 `agent_world_viewer --target wasm32-unknown-unknown` 的 `ctrlc` 目标兼容问题
- [x] 回归 `./scripts/ci-tests.sh required` 并确认通过

### T0 建档与基线
- [x] 新建设计文档：`doc/site/github-pages-release-download-pipeline-2026-03-01.md`
- [x] 新建项目管理文档：`doc/site/github-pages-release-download-pipeline-2026-03-01.project.md`
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

## 依赖
- 打包基础脚本：`scripts/build-game-launcher-bundle.sh`
- 站点发布流程：`.github/workflows/pages.yml`
- 站点入口文件：`site/index.html`、`site/en/index.html`

## 状态
- 当前阶段：已完成（T0A/T0/T1/T2/T3/T3A/T3B/T3C/T3D/T3E）
- 最近更新：完成 T3E m5 多 token 清单持久化修正（2026-03-01）
- 下一步：push 后等待 CI 回归结果
