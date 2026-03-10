# Viewer WebGL Deferred 兼容降级设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-webgl-deferred-compat-2026-02-24.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-webgl-deferred-compat-2026-02-24.project.md`

## 1. 设计定位
定义 wasm/WebGL2 环境下的 deferred lighting 兼容降级方案：避免 `copy_deferred_lighting_id_pipeline` 导致的致命崩溃，同时保持 forward 3D 主路径可用。

## 2. 设计结构
- 平台分流层：仅在 `target_arch = wasm32` 下调整 PBR 插件配置。
- 兼容降级层：关闭默认 deferred lighting 插件初始化。
- 行为保持层：native 路径保持不变，Web 继续走 forward 主路径。
- 验证收口层：native/wasm check 与 Playwright 3D 闭环共同确认“无致命崩溃”。

## 3. 关键接口 / 入口
- `app_bootstrap.rs`
- `PbrPlugin`
- `add_default_deferred_lighting_plugin = false`
- Playwright 3D 闭环

## 4. 约束与边界
- 只做 Web 路径兼容降级，不触碰 `third_party` 和模拟逻辑。
- 验收以“不再 panic”作为硬门禁，不把性能告警与致命崩溃混为一谈。
- native 视觉效果不能被 wasm 兼容策略误伤。
- 关闭 deferred 后的光照退化属于可接受代价。

## 5. 设计演进计划
- 先在 wasm 路径关闭 deferred lighting。
- 再做 native/wasm check。
- 最后通过 Web 3D 闭环确认崩溃消失。
