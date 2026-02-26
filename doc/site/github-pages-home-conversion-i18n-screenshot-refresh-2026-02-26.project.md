# GitHub Pages 首页转化文案与中英一致性收敛 + UI 截图刷新（2026-02-26）项目管理文档

## 任务拆解

### 0. 建档与基线
- [x] 新增设计文档（`doc/site/github-pages-home-conversion-i18n-screenshot-refresh-2026-02-26.md`）
- [x] 新增项目管理文档（本文件）
- [x] 明确改动范围为首页文案/一致性与三张场景截图

### 1. 首屏转化文案收敛（CN/EN）
- [x] 调整 `site/index.html` 首屏与上手区文案，避免超预期承诺
- [x] 调整 `site/en/index.html` 对应文案并与中文语义对齐
- [x] 保持锚点与交互标记兼容

### 2. 中英文一致性校准
- [x] 清理中文页关键区域混杂英文术语问题（保留必要专有名词）
- [x] 校验 CN/EN section 同构与导航语义一致

### 3. 游戏 UI 截图刷新
- [ ] 按 S6 Web 闭环重新采集 `minimal` 截图
- [ ] 按 S6 Web 闭环重新采集 `twin_region_bootstrap` 截图
- [ ] 按 S6 Web 闭环重新采集 `triad_region_bootstrap` 截图
- [ ] 替换 `site/assets/images/screenshots/*.webp`

### 4. 验证与收口
- [ ] 执行 `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer`
- [ ] 回写本项目管理文档状态
- [ ] 写任务日志（`doc/devlog/2026-02-26.md`）

## 依赖
- 站点结构：`site/` 静态目录与 `site/assets/app.js` 现有交互。
- 测试流程：`testing-manual.md` S6 Web 闭环。

## 状态
- 当前阶段：进行中（任务 0/1/2 已完成，任务 3/4 待完成）
- 最近更新：完成任务 2（中英文一致性校准）（2026-02-26）
- 下一步：执行任务 3（游戏 UI 截图刷新）。
