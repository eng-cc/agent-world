# GitHub Pages 用户视角问题修复（2026-02-26）设计文档

审计轮次: 4

- 审计轮次: 2

- 对应项目管理文档: doc/site/github-pages/github-pages-user-perspective-adjustments-2026-02-26.prd.project.md

## ROUND-002 主从口径
- 主入口：`doc/site/github-pages/github-pages-game-engine-reposition-2026-02-25.prd.md`
- 本文仅维护本专题增量，不重复主文档口径。

## 目标
- 修复首页顶部导航在长页面滚动时失效的问题，保证用户在任意位置都能快速导航与切换语言。
- 修复语言自动跳转状态写入时机，避免英文用户先访问文档页后被锁定在中文首页。
- 优化首页字体加载策略，降低英文用户首屏不必要的外部字体请求。
- 提升移动端菜单与语言按钮可点击面积，并补全菜单展开状态的无障碍语义。

## 范围
- 范围内
  - 更新 `site/assets/styles.css`（sticky 兼容、移动端点按热区、按钮尺寸）。
  - 更新 `site/assets/app.js`（语言重定向状态写入时机、菜单 aria 状态同步）。
  - 必要时更新 `site/index.html`、`site/en/index.html`、`site/doc/cn|en/*.html` 的导航语义标记。
- 范围外
  - 不调整首页信息架构与章节顺序。
  - 不改 Rust 业务逻辑。
  - 不引入新的前端构建工具链。

## 接口/数据
- 兼容约束
  - 保持 `data-menu`、`data-menu-toggle`、`data-lang-*`、`data-proof-*` 等标记兼容。
  - 保持现有锚点可用：`#matrix`、`#scenarios`、`#demo`、`#proof`、`#roadmap`、`#architecture`、`#contribute`。
- 行为变更
  - 顶部导航在滚动时持续可见。
  - 语言自动跳转仅在首页入口路径完成判定，不在 docs 路径提前写入完成标记。
  - 菜单按钮在移动端具备更大点击区域与 `aria-expanded` 同步。

## 里程碑
- M0：建档与任务拆解。
- M1：完成导航 sticky + 移动端可点击性/无障碍修复。
- M2：完成语言重定向时机与字体加载优化。
- M3：完成 Playwright 回归与 `cargo check` 验证，回写文档与日志。

## 风险
- 风险：去除横向 overflow 限制可能引入偶发横向滚动。
  - 缓解：以 Playwright 在 1280/900/360 宽度校验 `scrollWidth === clientWidth`。
- 风险：调整语言重定向逻辑后，已有用户本地状态行为变化。
  - 缓解：保持手动语言选择优先级不变，仅修正首次自动判定时机。
- 风险：字体策略调整影响中文页面观感一致性。
  - 缓解：保留中文系统字体 fallback，优先保障性能与可用性。

## 原文约束点映射（内容保真）
- 约束-1（目标与问题定义）：沿用原“目标”章节约束，不改变问题定义与解决方向。
- 约束-2（范围边界）：沿用原“范围”章节的 In Scope/Out of Scope 语义，不扩散到新增范围。
- 约束-3（接口/里程碑/风险）：沿用原接口字段、阶段节奏与风险口径，并保持可追溯。
