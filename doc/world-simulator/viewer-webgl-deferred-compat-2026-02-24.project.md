# Viewer WebGL Deferred 兼容降级项目管理文档（2026-02-24）

## 任务拆解
- [x] T0：设计文档与项目管理文档建档。
- [ ] T1：实现 wasm 路径关闭默认 deferred lighting 插件。
- [ ] T2：执行定向回归（native + wasm check、Playwright 3D 闭环）。
- [ ] T3：回写文档状态与 devlog，完成收口。

## 依赖
- T1 依赖 T0。
- T2 依赖 T1。
- T3 依赖 T2。

## 状态
- 当前阶段：T0 完成，T1 进行中。
- 阻塞项：无。
- 最近更新：2026-02-24。
