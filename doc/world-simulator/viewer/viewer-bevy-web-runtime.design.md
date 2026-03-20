# Viewer Bevy 浏览器运行路径设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-bevy-web-runtime.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-bevy-web-runtime.project.md`

## 1. 设计定位
定义 `oasis7_viewer` 在 wasm/web 目标下的运行路径、入口页面和闭环验证方式，使 Viewer Web 成为默认验证链路而 native 截图退为 fallback。

## 2. 设计结构
- wasm 兼容层：修复 `oasis7` 与 `oasis7_viewer` 的浏览器编译不兼容点。
- 页面入口层：通过 `trunk` 页面与 `run-viewer-web.sh` 提供标准 Web 入口。
- 手册对齐层：统一 `viewer-manual`、`AGENTS` 与脚本文档中的 Web 默认口径。
- 闭环验证层：使用 Playwright 执行 Web 默认闭环，native 截图保留为 fallback。

## 3. 关键接口 / 入口
- `cargo check --target wasm32-unknown-unknown`
- `crates/oasis7_viewer/index.html`
- `scripts/run-viewer-web.sh`
- `viewer-manual.md`
- Playwright 闭环

## 4. 约束与边界
- Web 路径必须保持 viewer 原有核心语义，不因浏览器目标退化为演示壳子。
- native 路径仍保留，但不再作为默认闭环入口。
- 本轮不强制实现完整在线协议桥接，只保障 Web 可运行与可验证。
- 相关文档口径必须统一，避免双主流程。

## 5. 设计演进计划
- 先修 wasm 编译与入口页面。
- 再补脚本与手册文档对齐。
- 最后用 Playwright 闭环验证 Web 默认链路稳定可用。
