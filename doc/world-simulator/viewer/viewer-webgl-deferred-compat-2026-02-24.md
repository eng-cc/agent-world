# Viewer WebGL Deferred 兼容降级（2026-02-24）

## 目标
- 解决 Web 3D 路径在当前环境（WebGL2 + SwiftShader）偶发的致命崩溃：
  - `wgpu error: Validation Error`
  - `Device::create_render_pipeline label = copy_deferred_lighting_id_pipeline`
- 保证 Playwright 3D 闭环可稳定执行，不因该管线创建失败导致 `RuntimeError: unreachable`。

## 范围
- 仅调整 `agent_world_viewer` 的 wasm 启动渲染插件配置。
- 不改动 world 模拟逻辑、协议、UI 引导流程语义。
- 不触碰 `third_party`。

## 接口/数据
- 模块：`crates/agent_world_viewer/src/app_bootstrap.rs`
- 方案：在 `target_arch = wasm32` 下显式配置 `PbrPlugin`：
  - `add_default_deferred_lighting_plugin = false`
  - 其余保持默认
- 预期影响：
  - Web 路径不再初始化默认 deferred lighting 管线，避免 `copy_deferred_lighting_id_pipeline` 触发崩溃。
  - 3D 保持 forward 主路径可用。

## 里程碑
- M0：完成设计与项目管理文档建档。
- M1：完成 wasm 渲染插件兼容降级代码改造。
- M2：完成 `cargo check`（native + wasm target）与 Playwright 3D 闭环验证。
- M3：回写文档状态与 devlog 收口。

## 风险
- 风险 1：关闭默认 deferred lighting 后，Web 3D 部分高级光照效果可能退化。
  - 缓解：仅在 wasm 生效；native 路径保持不变。
- 风险 2：当前环境仍可能有独立 WebGL 驱动告警（如 `ReadPixels` stall）。
  - 缓解：将“panic 消失”与“性能告警”分开验收，本任务只以“无致命崩溃”作为门禁。
