# Viewer Web 构建体积裁剪设计（2026-03-02）

- 对应需求文档: `doc/world-simulator/viewer/viewer-web-build-pruning-2026-03-02.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-web-build-pruning-2026-03-02.project.md`

## 1. 设计定位
定义 viewer wasm 路径裁剪边界、依赖收敛方式与体积验证口径。

## 2. 设计结构
- 模块裁剪层：按 wasm 目标剥离 runtime / live / non-web 模块。
- 依赖收敛层：把 native-only 依赖下沉到条件编译。
- 验证层：通过 wasm check、trunk build 与体积对比验证收效。

## 3. 关键接口 / 入口
- `oasis7` / `oasis7_viewer` 的 wasm 条件编译入口
- `cargo check --target wasm32-unknown-unknown` 与 `trunk build --release`
- 体积对比与依赖树观察命令

## 4. 约束与边界
- native 运行路径不得回归。
- wasm 不支持动作必须明确拒绝，不得 silent drop。
- 体积优化应优先通过编译边界与 feature 收敛实现。

## 5. 设计演进计划
- 先完成 Design 补齐与互链回写。
- 再沿项目管理文档推进实现与验证。
