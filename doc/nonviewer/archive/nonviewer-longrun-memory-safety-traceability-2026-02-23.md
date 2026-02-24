> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-24

# Non-Viewer 长稳运行内存安全与可追溯治理（2026-02-23）

## 目标
- 对 non-viewer 运行链路完成 6 项长稳风险治理：
  - 1) 网络命令/诊断无界内存风险。
  - 2) 节点共识动作入口无界负载风险。
  - 3) 共识内存历史线性增长风险。
  - 4) 运行时队列/日志内存膨胀风险。
  - 5) dead-letter/metrics 文件长期膨胀风险。
  - 6) wasm 磁盘缓存 `unsafe deserialize` 风险。
- 保持“可追溯”原则：
  - 运行态采用有界内存；
  - 历史数据通过可持久化与分段归档元数据保留追溯能力，避免仅依赖单机常驻内存。

## 范围

### In Scope
- `agent_world_net`:
  - libp2p 命令通道改为有界背压；
  - 诊断缓存（published/errors/listening_addrs）改为有界窗口。
- `agent_world_node`:
  - 共识动作提交入口增加 payload 与队列上限。
- `agent_world_consensus`:
  - Quorum/PoS 历史记录与 attestation history 增加有界保留策略。
- `agent_world` runtime:
  - pending action/effect/inflight 容量治理；
  - journal 内存保留策略（保留可追溯元信息）。
- `agent_world_consensus` dead-letter store:
  - dead-letter 与 delivery metrics 增加保留与压缩策略。
- `agent_world_wasm_executor`:
  - 移除基于 `unsafe deserialize` 的磁盘预编译模块加载路径，替换为安全路径。
- 文档与测试：
  - 更新项目文档与 devlog；
  - 补充 tier-required 覆盖。

### Out of Scope
- viewer 视觉/交互。
- 分布式协议重构（链上鉴权或共识协议大改）。
- 业务规则数值调优。

## 接口/数据
- 网络配置新增（`Libp2pNetworkConfig`）：
  - `command_buffer_capacity`（有界命令队列容量）。
  - `max_published_messages`、`max_error_messages`、`max_listening_addrs`（诊断窗口）。
- 节点配置新增（`NodeConfig`）：
  - `max_pending_consensus_actions`、`max_consensus_action_payload_bytes`。
- 共识策略新增：
  - Quorum/PoS 历史记录最大保留条数、PoS 单验证者历史保留条数。
- Runtime 内存策略新增：
  - pending/inflight/journal 的有界保留策略和统计元数据。
- Dead-letter 存储策略新增：
  - 每 world/node 保留上限与压缩触发阈值。
- WasmExecutor 配置收敛：
  - 磁盘缓存路径继续可用，但只走安全可验证加载路径。

## 里程碑
- M0：建档 + 任务拆解。
- M1：完成 1/2（入口背压与队列上限）。
- M2：完成 3/4（共识/运行时内存治理）。
- M3：完成 5/6（dead-letter 与 wasm cache 安全）。
- M4：回归测试 + 文档收口。

## 风险
- 有界策略可能引入拒绝/丢弃行为。
  - 缓解：返回明确错误、保留计数器与追溯元信息。
- 历史裁剪可能影响某些依赖“全量内存历史”的路径。
  - 缓解：仅裁剪已终态/可重建内容，并保留可持久化追溯元数据。
- I/O 压缩/持久化增加延迟。
  - 缓解：阈值触发 + 批量处理。
- 移除不安全磁盘预编译反序列化可能增加首次编译耗时。
  - 缓解：保留内存 LRU 缓存，磁盘缓存改安全格式。

## 当前状态
- 状态：已完成（2026-02-23）
- 已完成：M0、M1、M2、M3、M4
- 进行中：无
- 未开始：无
