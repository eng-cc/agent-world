> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-24

# Rust 超限文件拆分（第二轮，2026-02-23）

## 目标
- 将当前超过仓库约束（单个 Rust 文件 >1200 行）的重点文件降到阈值内。
- 采用低风险重构方式，不改变对外行为与协议语义。
- 本轮优先收口以下 5 个文件：
  - `crates/agent_world_distfs/src/challenge.rs`
  - `crates/agent_world_consensus/src/pos.rs`
  - `crates/agent_world_consensus/src/membership_recovery/replay.rs`
  - `crates/agent_world_consensus/src/membership_recovery/replay_archive_federated.rs`
  - `crates/agent_world_node/src/replication.rs`

## 范围
- In scope
  - 将上述文件中的内联 `#[cfg(test)] mod tests { ... }` 拆分为同名子目录下 `tests.rs`。
  - 原文件改为 `#[cfg(test)] mod tests;`，保持测试路径与断言语义不变。
  - 对每个任务执行定向测试并更新任务状态与 devlog。
- Out of scope
  - 非测试逻辑的业务重构。
  - 公共 API 变更、网络协议变更、数据结构字段变更。
  - `third_party/` 下任何代码调整。

## 接口 / 数据
- 对外接口：不变。
- 内部模块组织调整：
  - `foo.rs` 内测试模块迁移到 `foo/tests.rs`。
  - 保持 `use super::*;` 与原测试辅助函数调用路径一致。
- 不修改序列化格式、不修改签名校验规则、不修改共识/复制协议字段。

## 里程碑
- M1：完成文档与任务拆解（T0）。
- M2：完成 `challenge.rs` 与 `pos.rs` 拆分（T1/T2）。
- M3：完成 `replay.rs` 与 `replay_archive_federated.rs` 拆分（T3/T4）。
- M4：完成 `replication.rs` 拆分、回归与收口（T5/T6）。

## 风险
- 模块路径风险：`mod tests;` 路径解析错误导致编译失败。
- 可见性风险：测试迁移后漏掉 `use super::*;` 或辅助导入导致测试失败。
- 混合代码风险：`replay.rs` 测试块不在文件末尾，抽取范围错误会影响生产逻辑。
