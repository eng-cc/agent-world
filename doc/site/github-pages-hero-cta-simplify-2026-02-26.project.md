# GitHub Pages 首屏 CTA 收敛与文案校准（2026-02-26）项目管理文档

## 任务拆解

### 0. 建档与范围确认
- [x] 新增设计文档（`doc/site/github-pages-hero-cta-simplify-2026-02-26.md`）
- [x] 新增项目管理文档（本文件）
- [x] 明确改动范围仅为首页 Hero CTA（CN/EN）

### 1. Hero CTA 收敛与文案改写
- [ ] `site/index.html`：首屏 CTA 收敛为 2 个（主：试玩/演示；次：文档）
- [ ] `site/en/index.html`：首屏 CTA 收敛为 2 个（同构）
- [ ] 重写主按钮文案，去除“30 秒进入首局”承诺性措辞
- [ ] 执行 `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer`
- [ ] 回写项目管理文档状态与任务日志

## 依赖
- 复用现有 `site/assets/styles.css` 的 `.cta-row` 与 `.button` 样式能力，不新增样式依赖。
- 保持 `site/assets/app.js` 现有行为，不引入新脚本。

## 状态
- 当前阶段：进行中（任务 0 完成，任务 1 待执行）
- 最近更新：完成建档并进入实现阶段（2026-02-26）
- 下一步：执行任务 1（首页 CTA 收敛与文案改写）。
