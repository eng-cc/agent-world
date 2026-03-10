# Agent World Runtime：WASM SDK no_std 优先化设计

- 对应需求文档: `doc/world-runtime/wasm/wasm-sdk-no-std.prd.md`
- 对应项目管理文档: `doc/world-runtime/wasm/wasm-sdk-no-std.project.md`

## 1. 设计定位
定义 WASM SDK 的 `no_std` 优先化设计，降低模块依赖面并提升多目标兼容性。

## 2. 设计结构
- SDK 裁剪层：把 SDK 拆到 `no_std` 可用的核心能力。
- 兼容适配层：对需要 `std` 的功能提供可选适配。
- 类型约束层：统一在 `no_std` 环境下可用的数据结构与错误语义。
- 构建回归层：验证 wasm/嵌入式目标编译兼容性。

## 3. 关键接口 / 入口
- SDK `no_std` 核心接口
- 可选 `std` 适配层
- 通用错误/类型定义
- 构建兼容回归

## 4. 约束与边界
- 核心 SDK 应优先保持 `no_std` 纯净。
- 可选适配不得污染默认依赖面。
- 不在本专题重写全部 SDK 能力。

## 5. 设计演进计划
- 先切出 `no_std` 核心。
- 再补可选适配与类型收敛。
- 最后执行多目标构建回归。
