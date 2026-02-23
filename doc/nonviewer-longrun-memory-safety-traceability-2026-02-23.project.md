# Non-Viewer 长稳运行内存安全与可追溯治理（项目管理）

## 任务拆解

### T0 建档
- [x] 设计文档：`doc/nonviewer-longrun-memory-safety-traceability-2026-02-23.md`
- [x] 项目文档：`doc/nonviewer-longrun-memory-safety-traceability-2026-02-23.project.md`

### T1 网络与入口背压（对应 1/2）
- [x] `agent_world_net/libp2p_net`：命令通道有界化 + 诊断缓存有界化
- [x] `agent_world_node`：共识动作 payload/队列上限
- [x] 补充/更新单测

### T2 共识与运行时内存治理（对应 3/4）
- [ ] `agent_world_consensus`：Quorum/PoS 历史有界保留
- [ ] `agent_world` runtime：pending/inflight/journal 有界保留策略
- [ ] 补充/更新单测

### T3 Dead-letter 与 wasm cache 安全（对应 5/6）
- [ ] `membership_recovery/dead_letter`：保留上限 + 压缩
- [ ] `agent_world_wasm_executor`：移除 `unsafe deserialize` 依赖路径
- [ ] 补充/更新单测

### T4 收口
- [ ] 运行 required-tier 回归
- [ ] 更新设计/项目文档状态
- [ ] 更新 `doc/devlog/2026-02-23.md`

## 依赖
- `crates/agent_world_net/src/libp2p_net.rs`
- `crates/agent_world_node/src/node_runtime_core.rs`
- `crates/agent_world_node/src/types.rs`
- `crates/agent_world_consensus/src/quorum.rs`
- `crates/agent_world_consensus/src/pos.rs`
- `crates/agent_world/src/runtime/world/*`
- `crates/agent_world_consensus/src/membership_recovery/dead_letter.rs`
- `crates/agent_world_wasm_executor/src/lib.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1
- 进行中：T2
- 未开始：T3、T4
- 阻塞项：无
