# Viewer Web 构建体积裁剪 Phase 2 设计（2026-03-02）

- 对应需求文档: `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.project.md`

## 1. 设计定位
定义基于 feature 精细化与资源去内联的第二阶段 wasm 体积优化设计。

## 2. 设计结构
- 特性层：将 `bevy` 从默认特性切换为显式最小特性集。
- 资源层：把大字体等嵌入资源改为运行时加载。
- 兼容层：保持 native/wasm 两端字体链路与 UI 功能一致。

## 3. 关键接口 / 入口
- `oasis7_viewer/Cargo.toml` feature 配置
- `AssetServer` 资源加载链路
- UI runtime 与 main 入口的字体接线

## 4. 约束与边界
- wasm 端资源加载失败需有回退或错误可见性。
- 资源去内联不能破坏 release bundle 可用性。
- 体积优化必须伴随构建回归。

## 5. 设计演进计划
- 先完成 Design 补齐与互链回写。
- 再沿项目管理文档推进实现与验证。
