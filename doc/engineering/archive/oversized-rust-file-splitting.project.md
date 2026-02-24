> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-24

# Rust 超限文件拆分（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/engineering/archive/oversized-rust-file-splitting.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：拆分 `crates/agent_world/src/simulator/llm_agent.rs`（<=1200 行）并补定向回归
- [x] T2：拆分 `crates/agent_world/src/viewer/live.rs`（<=1200 行）并补定向回归
- [x] T3：统一回归（`cargo check` + 定向 tests）并回写文档/devlog

## 依赖
- T1 依赖 `llm_agent` 子模块文件可见性与 `impl` 组织正确。
- T2 依赖 `viewer/live` 子模块函数可见性与调用路径不变。
- T3 依赖 T1/T2 全部完成。

## 状态
- 当前阶段：已完成（T0/T1/T2/T3 全部完成）
- 阻塞项：无
- 下一步：无
