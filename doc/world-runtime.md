# Agent World Runtime：AOS 风格 world+agent 运行时（设计文档）

## 目标
- 在现有 `agent-world` 中实现一套 **world+agent 运行时**，借鉴 AgentOS 的关键优势：确定性、可审计、可回放、能力/政策边界、显式副作用与收据、受控升级；以**自由沙盒 + WASM 动态模块**作为基础能力。
- 让世界成为第一性：所有状态改变必须经由 **事件 → 规则校验 → 状态演化** 的统一路径，可追溯、可重放。
- 为后续规模化（多 Agent、高并发交互、长期运行）打下可演化的运行时基座。
- 允许 Agent 通过可治理的模块演化引入“新事物”（Rust → WASM），并保证审计/回放与能力边界。

## 技术参考（AgentOS）
- **AIR 风格控制面**：以结构化数据描述模块/计划/能力/政策；本项目以 manifest/patch 方式做简化对齐。
- **WASM reducers/pure modules**：确定性计算在沙箱模块内完成，避免内核承载复杂逻辑。
- **Effects/Receipts**：所有外部 I/O 必须显式声明并生成收据，纳入事件流审计。
- **Capability/Policy**：无环境授权，最小权限授权与策略审计。
- **Shadow → Approve → Apply**：受控升级流程作为系统演化的基本机制。
- **Minimal trusted base**：内核保持最小可信边界，复杂性外置到模块/适配器。

## 范围

### In Scope（V1）
- **确定性内核**：单线程 stepper，固定顺序处理事件，避免不可控并发。
- **事件溯源**：事件日志 + 快照；世界状态由事件重放导出。
- **显式副作用**：外部 I/O 只能以 Effect 意图表达，不可在 reducer 内直接执行。
- **Receipt 机制**：每个 effect 产生收据并写回事件流。
- **收据签名与校验**：收据需包含签名/校验信息，支持审计与回放一致性（V1 采用 HMAC‑SHA256）。
- **能力与政策**：capability grants + policy gate，限制 effect 的类型、范围与预算。
- **控制面（Manifest）**：以数据结构声明 reducers、effects、caps、policies、routing。
- **受控升级**：支持 propose → shadow → approve → apply 的最小治理流程（用于 manifest 级别变更）。
- **回滚与审计**：支持基于快照的回滚，并记录 RollbackApplied 审计事件。
- **Patch 与 Diff/Merge**：支持 manifest patch（set/remove）与 diff/merge 辅助函数。
- **冲突检测与快照清理**：merge 时报告冲突；快照保留策略可驱动文件清理。
- **可观测性**：事件尾随、收据查询、per-agent timeline。
- **文件级持久化**：journal/snapshot 的落盘与恢复接口。
- **WASM 模块沙箱（最小接入）**：模块以内容哈希登记，支持动态装载/调用；模块仅通过事件/Effect 与外部交互。

### Agent 机制（已选）
- **Agent = Cell（keyed reducer）**：每个 agent 由同一 reducer 的 keyed 实例表示（key = `agent_id`），拥有独立状态与 mailbox；事件路由按 key 分发，调度器以确定性顺序轮询。

### Out of Scope（V1 不做）
- 完整 AIR 规范与生产级 WASM 运行时实现（V1 仅保留简化版 manifest 与模块接口占位）。
- 跨 world 协议与一致性（后续阶段考虑）。
- 复杂并行执行（保持单线程确定性）。
- 完整 UI/可视化工具链（仅保留 CLI/日志接口）。

## 接口 / 数据

### 核心概念
- **World**：事件日志 + 快照 + manifest + reducer 状态集合。
- **Agent Cell**：同一 reducer 的 keyed 实例（`agent_id` 为 key）。
- **Reducer**：纯函数式状态机（输入事件 + 旧状态 → 新状态 + Effect 意图）。
- **WASM Module**：以 Rust 等语言编写并编译为 WASM 的可装载模块（reducer 或纯计算组件），运行在沙箱内。
- **Sandbox**：模块的受控执行环境，能力/政策约束在此生效，禁止直接 I/O。
- **Effect / Receipt**：显式副作用与其回执；重放时只读取 receipt，不重新执行 I/O。
- **Capability / Policy**：运行时授权与治理规则。


### 详细设计分册
- `doc/world-runtime/wasm-interface.md`：WASM 扩展接口、ABI/序列化、关键数据结构
- `doc/world-runtime/governance-events.md`：模块事件、ShadowReport、失败事件与治理事件负载
- `doc/world-runtime/audit-export.md`：审计导出记录、分页/分片、一致性与示例
- `doc/world-runtime/module-lifecycle.md`：模块治理与兼容性、注册表、治理流程、ModuleChangeSet、冲突与迁移
- `doc/world-runtime/runtime-integration.md`：模块加载/沙箱/路由/输出校验与运行时接口
- `doc/world-runtime/testing.md`：集成测试用例与测试基架建议
- `doc/world-runtime/module-storage.md`：模块存储持久化（registry/meta/artifacts）

## 里程碑
- **M0**：方案与接口冻结（本设计 + 项目管理文档）
- **M1**：确定性 world kernel + 事件日志 + 最小快照
- **M2**：Effect/Receipt 路径 + capability + policy gate
- **M3**：Agent cells + 调度器 + 基础可观测性
- **M4**：受控升级（propose/shadow/approve/apply）最小闭环
- **M5**：WASM 模块治理接入（注册表/路由/沙箱最小执行）

## 风险
- **“所有优点”带来的复杂度**：治理、收据、能力边界会显著增加实现成本。
- **确定性与性能冲突**：单线程+事件重放可能成为瓶颈。
- **持久化膨胀**：日志与收据增长快，需要快照与归档策略。
- **治理摩擦**：过严的审批/策略可能降低迭代速度。
