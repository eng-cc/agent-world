# GitHub Pages 惊艳化精修（二期）项目管理文档

## 任务拆解

### 0. 文档与基线
- [x] 输出二期设计文档（`doc/github-pages-wow-polish.md`）
- [x] 输出二期项目管理文档（本文件）
- [x] 建立发布前检查清单（视觉/内容/性能/可访问性）

### 1. 内容结构重构（双语同步）
- [x] 重写中文首页信息架构（Hero → Why → Matrix → Architecture → Scenarios → FAQ）
- [x] 重写英文首页同构信息架构（锚点与模块顺序一致）
- [x] 新增“实战场景”模块文案（至少 2 个）
- [x] 新增“质量与可信度”模块文案（测试/审计/可观测）
- [x] 新增“参与贡献”模块文案（Issue/PR/Discussion 路径）
- [x] 增补 FAQ（中文至少 6 条）
- [x] 增补 FAQ（英文至少 6 条）

### 2. 科技感视觉系统
- [x] 定义二期 CSS 设计 Token（颜色/阴影/排版/间距）
- [x] 升级首屏背景层（渐变 + 光效 + 网格/噪声轻纹理）
- [x] 升级卡片体系（hover、focus、active 状态统一）
- [x] 升级按钮体系（主次 CTA 对比度和点击反馈）
- [x] 升级 section 分割与滚动层次（避免视觉平铺）
- [x] 优化代码块视觉（更科技风、可读性更高）

### 3. 交互增强（轻量）
- [x] 增加滚动显隐（section reveal）
- [x] 增加导航激活态（当前锚点高亮）
- [x] 增加计数/指标渐进动画（支持降级）
- [x] 增加时间线交互（已完成/进行中节点态）
- [x] 完善移动端菜单交互细节（点击收起、滚动锁定策略）
- [x] 完善语言切换菜单可访问性（键盘焦点流）

### 4. 证据与素材增强
- [x] 按现有 viewer 流程产出首批真实截图素材（至少 4 张）
- [x] 统一素材压缩策略（WebP/SVG 优先）
- [x] 新增架构示意图（Runtime/Simulator/Viewer/LLM）
- [x] 新增分享封面图 `og-cover.png`
- [x] 更新首页模块配图与 caption（中英文一致）

### 5. SEO 与传播
- [x] 完善首页 meta title/description（中文）
- [x] 完善英文页 meta title/description（英文）
- [x] 增加 OpenGraph 与 Twitter Card 元信息
- [x] 增加 canonical 与语言替代链接（hreflang）

### 6. 质量门禁与回归
- [x] Playwright 桌面/移动端截图回归（中英文）
- [x] 关键锚点/外链 smoke check
- [x] Lighthouse Mobile 四项指标目标 >= 90
- [x] 资源体积检查（首屏关键资源可控）
- [x] 执行 `env -u RUSTC_WRAPPER cargo check`

### 7. 收尾
- [ ] 更新 `README.md` 展示站说明（补充二期亮点）
- [ ] 更新项目管理文档状态
- [ ] 写入当日开发日志（`doc/devlog/2026-02-10.md`）

## 依赖
- GitHub Pages 已启用且 `pages.yml` 持续可用。
- 可获取并生成 viewer 截图素材。
- 当前静态目录结构 `site/` 继续沿用，无需引入新构建工具。

## 状态
- 当前阶段：执行中（发布前技术与人工验收已完成）
- 最近更新：完成架构示意图（SVG）接入与素材增强任务全量收口（2026-02-10）
- 下一步：可按展示需要继续补充动态演示 GIF 或分场景专题页。
