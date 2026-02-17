# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 7）设计文档

## 目标
- 将 sequencer `accepted_action_signer_public_keys` 升级为生产级治理语义：配置合法性校验、规范化比较、重复项拒绝。
- 统一 ed25519 signer 公钥白名单行为，避免大小写/格式差异导致动作签名误拒绝。
- 保持现有协议兼容，不改变 action/head 线协议字段格式。

## 范围

### In Scope
- **HP7-1：Sequencer allowlist 校验与规范化**
  - 对 `SequencerMainloopConfig.accepted_action_signer_public_keys` 增加校验：
    - 不能为空字符串。
    - 必须是合法 32-byte hex ed25519 公钥。
    - 按小写规范化后去重，发现重复直接拒绝配置。
  - 保留 `require_action_signature` 语义：允许 `hmac_signer` 或 signer 公钥白名单任一满足。

- **HP7-2：动作验签路径接线**
  - `verify_action_signature` 中对签名提取出的 signer 公钥做规范化后再比较白名单。
  - 白名单匹配改为基于规范化集合，支持大小写无关。
  - 保持 HMAC 兼容路径不变。

- **HP7-3：测试与回归**
  - 新增/调整 sequencer 测试覆盖：
    - 非法 allowlist key 配置拒绝。
    - 规范化后重复 key 配置拒绝。
    - 白名单大写 key 与签名小写 key 匹配通过。
  - 执行 `agent_world_consensus` 与 `agent_world` `test_tier_required` 回归。

### Out of Scope
- 证书链、公钥托管、密钥轮换审批平台。
- `signature.rs` 对签名串返回值语义改造（本期仅在 sequencer 比较侧规范化）。
- 新增协议字段或修改 `ActionEnvelope` 数据结构。

## 接口 / 数据

### 配置语义
```rust
SequencerMainloopConfig {
  accepted_action_signer_public_keys: Vec<String>,
  // 其余字段保持不变
}
```

### 新行为
- 配置校验阶段：`accepted_action_signer_public_keys` 必须是 32-byte hex，规范化后不得重复。
- 运行比较阶段：签名中的 signer 公钥规范化后与 allowlist 集合比较（大小写无关）。

## 里程碑
- **HP7-M0**：设计文档 + 项目管理文档。
- **HP7-M1**：配置校验/规范化实现并接线。
- **HP7-M2**：测试回归与文档收口。

## 风险
- 严格校验会暴露历史脏配置，短期可能引发配置升级失败，需要明确错误口径。
- 若外部签名实现输出大写 hex，本期需确保比较侧规范化后可兼容。
- 去重策略改变后，原有包含重复 key 的配置会从“可运行”变为“启动失败”。
