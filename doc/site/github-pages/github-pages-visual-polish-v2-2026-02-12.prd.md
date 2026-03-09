# GitHub Pages 视觉细节打磨 V2（2026-02-12）设计文档

审计轮次: 5

- 对应项目管理文档: doc/site/github-pages/github-pages-visual-polish-v2-2026-02-12.project.md
- 对应标准执行入口: `doc/site/github-pages/github-pages-visual-polish-v2-2026-02-12.project.md`

## ROUND-002 主从口径
- 主入口：`doc/site/github-pages/github-pages-game-engine-reposition-2026-02-25.prd.md`
- 本文仅维护本专题增量，不重复主文档口径。

## 目标
- 在不重做结构的前提下，继续降低页面阅读负担，重点改善移动端信息密度。
- 强化 Hero 指标区与证据条的层级对比，让关键信息“先看懂、再细读”。
- 为路线图状态增加更直接的视觉编码（状态图标 + 轻量动态），提升科技感与状态可扫读性。

## 范围
- 范围内
  - 调整 `site/assets/styles.css` 的排版、间距与移动端密度策略。
  - 对中英文首页的证据条与路线图条目添加最小必要结构标记（class），用于样式增强。
  - 新增路线图状态轻量动画，并在 `prefers-reduced-motion` 下自动降级。
- 范围外
  - 不新增页面，不调整现有 section 信息架构。
  - 不引入第三方 UI 依赖，不改动现有脚本逻辑（除必要样式绑定）。

## 接口/数据
- 页面文件
  - `site/index.html`
  - `site/en/index.html`
- 样式文件
  - `site/assets/styles.css`
- 验证产物
  - `output/playwright/*.png`（中英文 + 桌面/移动截图）

## 里程碑
- M1：文档与任务拆解
  - 输出本设计文档与项目管理文档。
- M2：视觉实现
  - 完成三项优化：移动端密度、Hero/证据条层级、路线图状态编码。
- M3：验证收口
  - 截图回归验证 + `env -u RUSTC_WRAPPER cargo check` + 文档状态收口。

## 风险
- 风险：移动端压缩过度导致信息可读性下降。
  - 缓解：仅压缩次级文本，保留标题与关键 CTA 可读字号。
- 风险：状态动画带来干扰或性能抖动。
  - 缓解：使用低幅度动画并受 `prefers-reduced-motion` 控制。
- 风险：中英文页面样式挂钩不一致。
  - 缓解：采用统一 class 命名并双语同步改动。

## 原文约束点映射（内容保真）
- 约束-1（目标与问题定义）：沿用原“目标”章节约束，不改变问题定义与解决方向。
- 约束-2（范围边界）：沿用原“范围”章节的 In Scope/Out of Scope 语义，不扩散到新增范围。
- 约束-3（接口/里程碑/风险）：沿用原接口字段、阶段节奏与风险口径，并保持可追溯。
