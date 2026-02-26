# GitHub Pages 质量门禁 + 文档镜像同步 + SEO 元信息加固（2026-02-26）项目管理文档

## 任务拆解

### T0 建档与基线
- [x] 新建设计文档：`doc/site/github-pages-quality-gates-sync-seo-hardening-2026-02-26.md`
- [x] 新建项目管理文档：`doc/site/github-pages-quality-gates-sync-seo-hardening-2026-02-26.project.md`
- [x] 明确实施范围：Pages 门禁、手册镜像同步、SEO 元信息

### T1 手册镜像路径修复与一致性检查
- [x] 修复 `site/doc/cn/viewer-manual.html` 过时 Playwright 路径
- [x] 修复 `site/doc/en/viewer-manual.html` 过时 Playwright 路径
- [x] 新增 `scripts/site-manual-sync-check.sh`（关键口径一致性校验）
- [x] 任务测试与提交

### T2 Pages 发布门禁接入
- [ ] 新增 `scripts/site-link-check.sh`（站内相对链接校验）
- [ ] 更新 `.github/workflows/pages.yml`，部署前执行校验脚本
- [ ] 任务测试与提交

### T3 首页 SEO 元信息加固
- [ ] 更新 `site/index.html` SEO/社交元信息
- [ ] 更新 `site/en/index.html` SEO/社交元信息
- [ ] 校验 canonical/hreflang 与 OG/Twitter 字段一致性
- [ ] 任务测试与提交

### T4 回归验证与文档收口
- [ ] 执行站点脚本回归校验与页面冒烟
- [ ] 回写本项目管理文档状态
- [ ] 写任务日志：`doc/devlog/2026-02-26.md`
- [ ] 任务测试与提交

## 依赖
- 站点静态结构：`site/` 与 `site/assets/app.js` 既有交互契约。
- 发布流程：`.github/workflows/pages.yml`。
- 手册内容基线：`doc/viewer-manual.md`。

## 状态
- 当前阶段：进行中（T0/T1 完成，T2 进行中）
- 最近更新：完成 T1（手册镜像路径修复与一致性检查脚本）（2026-02-26）
- 下一步：执行 T2，新增站内链接检查并接入 Pages 发布门禁。
