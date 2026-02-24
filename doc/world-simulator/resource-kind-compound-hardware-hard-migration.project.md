# Compound/Hardware 硬迁移：从内建 ResourceKind 移除并转向 WASM 定义（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/world-simulator/resource-kind-compound-hardware-hard-migration.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：代码硬迁移（移除 `ResourceKind::Compound/Hardware`，同步解析/viewer/README）
- [x] T2：修复测试与门禁回归，回写项目状态与 devlog

## 依赖
- T1 依赖 T0 先明确“无兼容方案”的边界。
- T2 依赖 T1 完成后统一回归。

## 状态
- 当前阶段：T0/T1/T2 已完成。
- 阻塞项：无。
- 下一步：无。
