# Non-Viewer 设计一致性审查 Round2（2026-02-25）

## 目标
- 在已完成 `doc/nonviewer/nonviewer-design-alignment-fixes-2026-02-25.md` 的基础上，继续执行第二轮 non-viewer 设计一致性审查。
- 覆盖非 Viewer 的活跃设计分册（优先 `testing` / `p2p` / `world-runtime`），识别“设计要求与实现行为不一致”项。
- 对确认问题统一收敛优化，并同步文档与任务日志。

## 范围

### In Scope
- 新增并维护第二轮审查记录：问题台账、核对范围、结论。
- 记录并跟踪已发现问题：non-viewer Rust 单文件超过 1200 行的工程约束偏差。
- 核对非 Viewer 代码实现与设计文档的一致性（含必要定向测试）。
- 对本轮确认问题执行一并优化（代码/文档）。

### Out of Scope
- `crates/agent_world_viewer` 及 Viewer UI/Web 交互代码。
- 仅归档文档的历史一致性回溯重审。

## 接口 / 数据
- 问题台账字段：`id` / `scope` / `design_source` / `implementation` / `status` / `notes`。
- 约束项：
  - Rust 文件行数上限：1200（以仓库工作流约束为准）。
  - 设计一致性判定：文档声明行为与运行时实际行为/测试结果一致。
- 输出物：
  - 本设计文档与项目管理文档状态更新。
  - 任务日志（`doc/devlog/2026-02-25.md`）。

## 里程碑
- M0：建档并记录已发现问题。
- M1：完成第二轮审查（testing/p2p/world-runtime 优先路径）。
- M2：完成批量优化与回归测试。
- M3：回写文档状态与 devlog 收口。

## 风险
- 审查范围较大，存在漏检风险。
  - 缓解：优先活跃分册与近期变更路径，按“文档 -> 代码 -> 测试”闭环核对。
- 批量优化可能引入回归。
  - 缓解：每项优化配套定向测试与编译校验。

## 第二轮问题清单与优化结果
- R2-01（工程约束，high）
  - design_source：`AGENTS.md`（Rust 单文件不超过 1200 行）。
  - implementation：
    - `crates/agent_world/src/simulator/world_model.rs`（1244）
    - `crates/agent_world/src/simulator/kernel/actions_impl_part1.rs`（1236）
    - `crates/agent_world_consensus/src/quorum.rs`（1233）
  - status：已修复
  - notes：
    - `world_model.rs` 抽离 `PhysicsParameterSpec` 与规格常量到 `world_model/world_model_physics_specs.rs`。
    - `actions_impl_part1.rs` 将 `BuildFactory`/`ScheduleRecipe` 分支下沉为 `part2` 私有方法。
    - `quorum.rs` 测试模块拆分至 `quorum/tests.rs`。
- R2-02（设计一致性，high）
  - design_source：`doc/p2p/p2p-blockchain-security-hardening-2026-02-23.md`（订阅队列有界）。
  - implementation：`crates/agent_world_consensus/src/network.rs` 在 `publish` 中直接向 `Vec` 追加，存在无界增长风险。
  - status：已修复
  - notes：改为复用 `agent_world_proto::distributed_net::push_bounded_inbox_message`，并新增超限淘汰最旧消息单测。

## 当前状态
- 状态：已完成（2026-02-25）
- 已完成：M0、M1、M2、M3
- 进行中：无
- 阻塞项：无
