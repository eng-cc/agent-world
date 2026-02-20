> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Node 主循环基础模块（设计文档）

## 目标
- 新增独立 `node` crate，提供可复用的节点主循环基础能力，供 `agent_world` 在启动模拟时直接调用。
- 将“节点生命周期控制”从业务入口中抽离，统一为 `NodeRuntime` 接口（启动、停止、状态快照）。
- 先落地最小闭环：单进程可启动一个逻辑节点并周期运行 tick，为后续接入网络/共识执行路径预留扩展点。

## 范围

### In Scope
- 新增 `crates/agent_world_node`，包含：
  - `NodeRole`：节点角色枚举（`sequencer` / `storage` / `observer`）。
  - `NodeConfig`：节点配置（`node_id` / `world_id` / `tick_interval` / `role`）。
  - `NodeRuntime`：节点主循环生命周期接口（`start` / `stop` / `snapshot`）。
  - `NodeSnapshot`：运行态快照（是否运行、tick 计数、最后 tick 时间）。
- 在 `world_viewer_live` 启动流程中默认启动一个节点实例，并支持 CLI 配置。
- 补齐单元测试与参数解析测试，保证可测闭环。

### Out of Scope
- 多节点真实网络同步与跨进程消息分发。
- 共识提案/投票执行与区块提交闭环。
- 持久化节点状态恢复（重启续跑）。
- 运维面板与可视化接线。

## 接口 / 数据

### NodeRole
- `Sequencer`
- `Storage`
- `Observer`

### NodeConfig
- `node_id: String`
- `world_id: String`
- `tick_interval: Duration`
- `role: NodeRole`

### NodeRuntime
- `new(config) -> NodeRuntime`
- `start() -> Result<(), NodeError>`
- `stop() -> Result<(), NodeError>`
- `snapshot() -> NodeSnapshot`

### NodeSnapshot
- `running: bool`
- `tick_count: u64`
- `last_tick_unix_ms: Option<i64>`

## 里程碑
- DNM-1：设计文档与项目管理文档落地。
- DNM-2：`node` crate 基础类型与主循环实现。
- DNM-3：`agent_world` 启动模拟时接入节点自动启动。
- DNM-4：测试回归、文档状态收口与 devlog。

## 风险
- 当前主循环仅提供最小 tick 能力，未承载真实网络/共识逻辑，后续需要平滑扩展接口。
- 若入口默认启动节点，需注意未来多角色组合时的参数复杂度与可观测性。
- 节点运行线程与主服务线程并行，需确保停止流程可靠，避免测试残留线程。
