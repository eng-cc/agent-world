# GitHub Pages 首页转化文案与中英一致性收敛 + UI 截图刷新（2026-02-26）设计文档

## 目标
- 收敛首页首屏转化承诺，确保“点击预期”与真实上手门槛一致，降低新用户落差。
- 校准首页中英文表达一致性，避免中文页混入过多英文术语导致认知断层。
- 基于最新游戏 UI 刷新首页场景截图，确保展示素材与当前版本一致。

## 范围
- 范围内
  - 更新首页文案：`site/index.html`、`site/en/index.html`。
  - 校准中英文同构语义（保持 section 顺序、锚点、`data-*` 兼容）。
  - 重新生成并替换首页场景截图：
    - `site/assets/images/screenshots/minimal.webp`
    - `site/assets/images/screenshots/twin_region_bootstrap.webp`
    - `site/assets/images/screenshots/triad_region_bootstrap.webp`
- 范围外
  - 不改 `site/doc/cn|en/*.html` 手册正文结构。
  - 不改 Rust 运行时/Viewer 功能逻辑。
  - 不引入新的前端构建链路。

## 接口/数据
- 页面兼容约束
  - 保持 `site/assets/app.js` 依赖的锚点与标记可用：`#matrix`、`#scenarios`、`#demo`、`#proof`、`#roadmap`、`#architecture`、`#contribute`。
  - 保持 `data-reveal`、`data-counter-target`、`data-menu*`、`data-lang*`、`data-proof-*` 不变。
- 截图生成链路
  - 采用 Web 闭环（S6）作为默认：`world_viewer_live` + `run-viewer-web.sh` + Playwright。
  - 参考文档：`testing-manual.md`（S6 及补充约定）。

## 里程碑
- M0：建档与任务拆解。
- M1：完成首页首屏转化文案收敛与中英一致性校准。
- M2：完成三张场景图刷新并替换入站点素材目录。
- M3：完成验证、文档回写与 devlog 收口。

## 风险
- 风险：中英文文案在局部术语上继续漂移。
  - 缓解：CN/EN 同任务同步修改，逐段对齐语义。
- 风险：截图刷新流程受 Web 编译或运行态波动影响。
  - 缓解：严格走 S6 链路并保留产物用于复查。
- 风险：新截图体积上升影响首屏性能。
  - 缓解：保持 WebP 压缩与原文件名复用，避免额外请求。
