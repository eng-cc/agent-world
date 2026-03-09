# README 收口：基础设施模块执行引擎 + 编译 Sandbox 隔离（项目管理文档）

审计轮次: 4

## 审计备注
- 主项目入口：`doc/readme/gap/readme-gap-distributed-prod-hardening-gap12345.project.md`。
- 本文件仅维护本专题增量任务。

## 任务拆解
- [x] T0：输出设计文档（`doc/readme/gap/readme-gap-infra-exec-compiler-sandbox.prd.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：实现基础设施模块执行引擎
  - `WorldState` 持久化安装目标
  - `ModuleInstalled` 事件应用更新目标
  - tick 路由输出 `infrastructure_tick` 语义
  - required tests
- [x] T2：实现编译 Sandbox 隔离
  - 源码包约束（数量/大小/总量）
  - 编译进程 timeout
  - 环境变量最小化与隔离临时目录
  - required tests
- [x] T3：回归验证与收口
  - `env -u RUSTC_WRAPPER cargo check`
  - `CI_VERBOSE=1 ./scripts/ci-tests.sh required`
  - 项目文档状态 + devlog 回写

## 依赖
- T1 与 T2 可并行，但本轮按用户要求顺序执行：先 T1 再 T2。
- T3 依赖 T1/T2 完成后统一执行。

## 状态
- 当前阶段：已完成（T0/T1/T2/T3 全部完成）。
- 阻塞项：无。
- 下一步：无。

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
