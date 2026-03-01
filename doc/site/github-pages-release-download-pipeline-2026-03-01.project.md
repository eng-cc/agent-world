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

## 依赖
- 打包基础脚本：`scripts/build-game-launcher-bundle.sh`
- 站点发布流程：`.github/workflows/pages.yml`
- 站点入口文件：`site/index.html`、`site/en/index.html`

## 状态
- 当前阶段：已完成（T0A/T0/T1/T2/T3）
- 最近更新：完成 T3 回归验证与文档收口（2026-03-01）
- 下一步：无（本项目结项）
