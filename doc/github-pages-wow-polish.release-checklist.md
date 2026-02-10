# GitHub Pages 惊艳化精修（二期）发布前检查清单

> 最近执行日期：2026-02-10

## 1) 视觉与内容
- [x] 中英文页面信息架构一致（Why/Matrix/Architecture/Scenarios/Demo/Roadmap/Quality/FAQ/Contribute）。
- [x] Hero、卡片、按钮、代码块、时间线等核心模块样式已统一。
- [x] 架构示意图（Runtime/Simulator/Viewer/LLM）已接入中英文架构模块。
- [x] 移动端与桌面端截图回归通过（截图在 `.tmp/screens/`）。

## 2) 交互与可访问性
- [x] section reveal / nav active / 指标计数动画正常。
- [x] 时间线筛选交互（all/done/active/next）可用。
- [x] 语言菜单支持键盘焦点流（Enter/Space/ArrowUp/ArrowDown/Escape）。

## 3) SEO 与基础元信息
- [x] 中英文页均有 `meta description`。
- [x] 中英文页均有 OpenGraph/Twitter Card。
- [x] 中英文页均有 `canonical` 与 `hreflang`。

## 4) 质量门禁（本地）
- [x] Lighthouse（本地 `http://127.0.0.1:4180`）
  - 中文：Performance 100 / Accessibility 100 / Best Practices 100 / SEO 100
  - 英文：Performance 100 / Accessibility 100 / Best Practices 100 / SEO 100
  - 原始输出：`.tmp/lighthouse/zh-report`、`.tmp/lighthouse/en-report`
- [x] 锚点 smoke check（中英文均无无效锚点）。
- [x] 外链 smoke check（GitHub 相关链接返回 200）：`.tmp/lighthouse/link-smoke.log`

## 5) 资源体积预算（静态资源）
- [x] CSS `site/assets/styles.css`：16123B（预算 <= 20480B）
- [x] JS `site/assets/app.js`：11060B（预算 <= 15360B）
- [x] 核心 SVG `site/assets/images/world-loop.svg`：3868B（预算 <= 10240B）
- [x] 场景截图采用 WebP 压缩（4 张，总计约 269KB）。
- [x] 分享封面图 `site/assets/images/og-cover.png` 约 32KB。
- [x] 检查记录：`.tmp/lighthouse/resource-budget-v6.txt`

## 6) 工程校验
- [x] `env -u RUSTC_WRAPPER cargo check` 通过。

## 7) 上线前人工确认
- [x] GitHub Pages Settings 中 Source 已设为 GitHub Actions。
- [x] 线上域名（`https://eng-cc.github.io/agent-world/`）首屏与双语切换行为人工验收。
