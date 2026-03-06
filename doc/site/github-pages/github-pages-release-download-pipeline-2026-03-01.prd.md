# GitHub Pages 发布入口 + Release 安装包流水线（2026-03-01）设计文档

审计轮次: 3

- 审计轮次: 2

- 对应项目管理文档: doc/site/github-pages/github-pages-release-download-pipeline-2026-03-01.prd.project.md

## ROUND-002 主从口径
- 主入口：`doc/site/github-pages/github-pages-game-engine-reposition-2026-02-25.prd.md`
- 本文仅维护本专题增量，不重复主文档口径。

## 目标
- 建立可直接上线的发布系统：GitHub Pages 作为发行入口页，GitHub Releases 作为安装包分发源。
- 支持用户在官网页面“一键下载”最新安装包（Windows/macOS/Linux）。
- 将“打包 + 发布 + 校验”纳入 GitHub Actions，减少手工发布步骤和失误率。

## 范围
- 范围内
  - 新增 Release 发布工作流（tag 触发 + 手动触发）。
  - 自动构建桌面启动器安装包并上传到 GitHub Release。
  - 生成并上传校验文件（SHA256）。
  - 在 `site/index.html` 与 `site/en/index.html` 增加下载入口和直链（`releases/latest/download/...`）。
  - 站点脚本补充下载入口的存在性/基本格式校验。
- 范围外
  - 不改动 Rust 世界规则与游戏逻辑。
  - 不引入新的前端构建工具链。
  - 不实现应用内自动更新器（auto-updater）。

## 接口 / 数据
- Release 产物命名（固定名，保证 `latest/download` 可长期使用）：
  - `agent-world-windows-x64.zip`
  - `agent-world-macos-x64.tar.gz`
  - `agent-world-linux-x64.tar.gz`
  - `agent-world-checksums.txt`
- 下载直链：
  - `https://github.com/<owner>/<repo>/releases/latest/download/<asset>`
- 工作流触发：
  - `push tags: v*`
  - `workflow_dispatch`
- 打包 runner 与目标三元组（release workflow）：
  - linux：`ubuntu-24.04` + `native`
  - macOS：`macos-14` + `x86_64-apple-darwin`（避免仓库不支持的 `macos-13` 配置）
  - windows：`windows-2022` + `native`
- 打包内容（每个平台）：
  - `bin/world_game_launcher`
  - `bin/world_viewer_live`
  - `bin/world_chain_runtime`
  - `bin/agent_world_client_launcher`
  - `web/`（viewer 静态资源）
  - `run-game.sh` / `run-client.sh`（Windows 额外提供 `.cmd`）
  - `README.txt`

## 里程碑
- M0：建档（设计 + 项目管理）。
- M1：发布流水线可产出三平台安装包并写入 Release。
- M2：Pages 首页接入下载入口并直连 latest release assets。
- M3：完成校验、文档回写、devlog 记录与结项。

## 风险
- 风险：跨平台构建在 GitHub Runner 上依赖差异较大，可能导致单平台失败。
  - 缓解：Web 资源单独构建后复用；native 构建采用矩阵分离，失败平台可独立定位。
- 风险：固定资产名若被误改，页面直链会失效。
  - 缓解：新增下载入口校验脚本并接入 CI。
- 风险：`latest` 语义受 prerelease 影响，用户可能下载到非稳定版。
  - 缓解：工作流默认发布正式 release，必要时在文档中要求 prerelease 另行命名与渠道区分。

## 原文约束点映射（内容保真）
- 约束-1（目标与问题定义）：沿用原“目标”章节约束，不改变问题定义与解决方向。
- 约束-2（范围边界）：沿用原“范围”章节的 In Scope/Out of Scope 语义，不扩散到新增范围。
- 约束-3（接口/里程碑/风险）：沿用原接口字段、阶段节奏与风险口径，并保持可追溯。
