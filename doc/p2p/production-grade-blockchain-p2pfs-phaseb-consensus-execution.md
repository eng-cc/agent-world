# Agent World Runtime：生产级区块链 + P2P FS 路线图 Phase B（共识内生执行）

## 目标
- 将当前“reward runtime 外围 execution bridge 驱动执行”的模式推进为“节点共识主循环内生执行”。
- 让共识提交高度与执行高度/状态根在同一条节点主链路中推进，降低双循环一致性风险。
- 保持与现有报表、CAS 记录目录兼容，并提供平滑 fallback。

## 范围

### In Scope（本轮）
- **PRG-B1：节点执行 Hook 接口与快照字段**
  - 在 `agent_world_node` 引入可注入的执行驱动接口（commit 后触发）。
  - `NodeConsensusSnapshot` 增加 execution 相关字段（执行高度、执行块哈希、执行状态根）。
  - 运行时重启后恢复 execution 快照字段。

- **PRG-B2：world_viewer_live 内生执行接线**
  - 新增 `NodeRuntimeExecutionDriver`，将现有 execution bridge 的步进/落盘逻辑封装为节点执行驱动。
  - 在 `start_live_node` 启动时注入到 `NodeRuntime`。
  - reward runtime 循环优先读取节点内生执行产物；仅在未启用内生执行时走旧 bridge fallback。

- **PRG-B3：测试与回归**
  - 在 `agent_world_node` 增加 execution hook 行为测试（提交触发、持久化恢复）。
  - 在 `world_viewer_live` 增加 execution driver 单测（执行记录落盘与哈希链）。
  - 执行 `test_tier_required` 口径回归。

### Out of Scope（后续）
- DistFS challenge-response 跨节点请求/应答协议化（PRG-M5）。
- 需求侧交易撮合、报价和订单清结算（PRG-M6）。
- 治理层签名阈值升级与密钥轮换编排。

## 接口 / 数据

### 1) 节点执行驱动输入（新增）
```rust
NodeExecutionCommitContext {
  world_id: String,
  node_id: String,
  height: u64,
  slot: u64,
  epoch: u64,
  node_block_hash: String,
  committed_at_unix_ms: i64,
}
```

### 2) 节点执行结果（新增）
```rust
NodeExecutionCommitResult {
  execution_height: u64,
  execution_block_hash: String,
  execution_state_root: String,
}
```

### 3) 节点共识快照字段扩展（新增）
```rust
NodeConsensusSnapshot {
  // existing fields...
  last_execution_height: u64,
  last_execution_block_hash: Option<String>,
  last_execution_state_root: Option<String>,
}
```

### 4) 快照持久化扩展（新增）
```rust
PosNodeStateSnapshot {
  // existing fields...
  last_execution_height: u64,
  last_execution_block_hash: Option<String>,
  last_execution_state_root: Option<String>,
}
```

## 里程碑
- **PRG-BM0**：完成 Phase B 设计文档与项目管理文档。
- **PRG-BM1**：`agent_world_node` 执行 hook + 快照/持久化扩展。
- **PRG-BM2**：`world_viewer_live` execution driver 接入 NodeRuntime。
- **PRG-BM3**：测试回归、文档与 devlog 收口。

## 风险
- 节点线程内执行耗时若过长，可能影响共识 tick 周期；需保持执行驱动幂等且轻量。
- 内生执行与旧 bridge 并存阶段，若切换条件不清晰可能导致重复执行。
- 快照字段扩展需要 `serde(default)` 兼容旧状态文件，避免升级失败。
