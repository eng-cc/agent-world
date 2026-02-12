# Agent World Runtime：`agent_world_net` + `agent_world_consensus` 拆分（设计文档）

## 目标
- 将分布式能力按职责拆为两个 crate：
  - `agent_world_net`：网络与传输相关能力。
  - `agent_world_consensus`：共识与成员同步相关能力。
- 明确将 `distributed_membership_sync` 归类到 `agent_world_consensus`。
- 保持 `agent_world` 对外 API 兼容，避免一次性大范围破坏。

## 范围

### In Scope（本次）
- 新建 crate：
  - `crates/agent_world_net`
  - `crates/agent_world_consensus`
- 将两个 crate 加入 workspace，并形成稳定导出入口（按能力分类导出）。
- `agent_world_consensus` 对外聚合共识与 membership sync 能力（含 `distributed_membership_sync` 相关导出）。
- 补充最小验证（编译/测试）与文档回写。

### Out of Scope（本次不做）
- 不在本轮强制把 `agent_world` 现有 runtime 实现文件全部物理迁移到新 crate。
- 不做协议层额外重构（协议仍以 `agent_world_proto` 为主）。
- 不改动业务语义与现有行为策略。

## 接口 / 数据

### `agent_world_net`（边界）
- 负责导出：
  - 网络抽象与 in-memory 网络实现。
  - 分布式网络客户端/网关/观察者等网络链路能力。
  - 可选 `libp2p` 能力导出（feature 透传）。

### `agent_world_consensus`（边界）
- 负责导出：
  - 共识主流程（提案/投票/提交）相关能力。
  - 成员目录同步与恢复能力。
  - 吊销对账、调度、告警、恢复、审计相关能力。
- `distributed_membership_sync` 及其子模块归属此 crate 的能力面。

## 里程碑
- P1：设计文档与项目管理文档完成。
- P2：新 crate 脚手架与 workspace 接线完成。
- P3：导出能力面落地（net/consensus 分类稳定）。
- P4：编译与定向测试回归通过，项目文档收口。

## 风险
- 仅做边界导出时，可能出现“新 crate 已存在但实现仍在 `agent_world`”的过渡期认知偏差。
- 导出项过多时，维护成本上升；需要后续按使用频率做分组收敛。
- feature 透传（尤其 `libp2p`）若配置不完整，可能导致构建行为与预期不一致。
