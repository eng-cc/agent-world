# Node 共识签名身份绑定与复制摄取硬化（设计文档）

## 目标
- 收口 P0：把 node 共识消息签名从“仅校验签名有效”升级为“校验签名有效 + 校验签名公钥与 validator 身份绑定一致”，阻断伪造 validator 身份的空间。
- 收口 P1-1：把 replication 入站处理从“先更新网络高度再尝试落库、且落库错误被吞”升级为“先成功校验/落库再更新观测高度，错误可观测”。
- 收口 P1-2：把启动阶段 PoS 状态恢复从“加载失败静默忽略”升级为“加载失败立即报错并阻止启动”。

## 范围

### In Scope
- `crates/agent_world_node/src/types.rs`
  - 为 `NodePosConfig` 增加 validator->signer 公钥绑定配置。
- `crates/agent_world_node/src/pos_validation.rs`
  - 增加 signer 公钥格式与归一化校验。
  - 校验 signer 绑定完整性（启用时要求与 validator 集一致）。
- `crates/agent_world_node/src/lib.rs`
  - `PosNodeEngine` 持有 signer 绑定表。
  - 共识消息 ingest 路径增加 signer 绑定校验。
  - replication ingest 调整为“先 apply 成功再 observe”。
  - replication ingest 错误改为可观测上抛（不再静默吞掉）。
  - 启动阶段 PoS 状态加载失败显式返回错误。
- `crates/agent_world_node/src/tests_*.rs`
  - 新增/更新 P0 与 P1 相关测试。

### Out of Scope
- 不重写共识协议，不引入新密码学算法。
- 不改动 `agent_world_consensus` 的签名策略配置模型。
- 不引入外部 KMS 或证书系统。

## 接口 / 数据

### 1) Validator Signer 绑定
- `NodePosConfig` 新增字段：
  - `validator_signer_public_keys: BTreeMap<String, String>`
- 语义：
  - key: `validator_id`
  - value: ed25519 公钥 hex（32-byte，允许大小写输入，内部归一化为小写）
- 校验规则：
  - 若 map 为空：保持兼容，不强制 signer 绑定。
  - 若 map 非空：必须覆盖全部 validator，且不能包含未知 validator。
  - map 值必须是合法 32-byte hex。

### 2) 共识消息验签增强
- 对 `proposal/attestation/commit`：
  1. 先执行现有签名校验。
  2. 再执行 `validator_id -> signer_public_key` 绑定校验。
- 绑定启用时（`validator_signer_public_keys` 非空）：
  - 消息必须携带 `public_key_hex`，且归一化后与配置绑定值一致。
  - 不一致则丢弃该消息。

### 3) Replication ingest 顺序与错误模型
- `ingest_network_replications` 行为调整：
  - 仅在 `apply_remote_message` 成功后，才更新 `network_committed_height/peer_heads` 与本地同步推进。
  - `apply_remote_message` 失败不再静默，转化为 `NodeError::Replication` 上抛（聚合错误摘要）。

### 4) 启动恢复错误显式化
- `NodeRuntime::start` 中 `PosNodeStateStore::load`：
  - `Err` 直接返回启动失败。
  - 禁止继续以“默认状态”悄然启动。

## 里程碑
- M1：T0 文档建档（本设计 + 项管）。
- M2：T1 完成 P0 数据模型与校验链路改造。
- M3：T2 完成 P0 测试闭环。
- M4：T3 完成 P1 replication ingest 与启动恢复硬化。
- M5：T4 完成 P1 测试与回归收口。

## 风险
- 兼容风险：启用 signer 绑定后，旧节点若未配置公钥映射将无法享受新防护，需要明确默认兼容策略。
- 运维风险：错误配置 signer map 会导致消息被拒，需要在配置校验阶段尽早失败并给出明确信息。
- 可用性风险：上抛 replication 错误会提升噪音，需要聚合错误文本避免日志风暴。
