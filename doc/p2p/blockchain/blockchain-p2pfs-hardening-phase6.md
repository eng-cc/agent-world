# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 6）设计文档

## 目标
- 在 Phase 5 signer 公钥白名单治理基础上，补齐**生产可运维**所需的策略配置校验能力。
- 将 `accepted_signature_signer_public_keys` 从“原样字符串比较”升级为“规范化后比较”，降低大小写与格式差异导致的误拒绝风险。
- 对策略误配（空白、非法 hex、长度错误、重复键）实现 fail-fast，避免在运行时静默退化。

## 范围

### In Scope
- **HP6-1：策略校验与规范化实现**
  - 新增 membership policy 校验入口：
    - `validate_membership_snapshot_restore_policy(...)`
    - `validate_membership_revocation_sync_policy(...)`
  - 对 `accepted_signature_signer_public_keys` 做统一规范化：
    - trim 后不能为空。
    - 必须可解码为 hex。
    - 必须是 32-byte ed25519 公钥。
    - 规范化为小写 hex，并按规范化值去重。
  - 签名提取路径输出规范化 signer 公钥，小写比较，避免大小写导致的误判。

- **HP6-2：校验接线与错误口径**
  - 在 membership restore/revocation 的 policy 驱动入口处先执行策略校验。
  - 策略不合法时直接返回明确错误，不进入后续同步/恢复流程。
  - 保持协议字段兼容，不新增线协议字段。

- **HP6-3：测试与回归**
  - 新增测试覆盖：
    - 非法 signer 公钥配置被拒绝。
    - 大小写混用白名单可正常匹配签名 signer。
    - 规范化后重复 signer 公钥配置被拒绝。
  - 执行 `agent_world_consensus` 与 `agent_world` `test_tier_required` 回归。

### Out of Scope
- CA/证书链、公钥托管、轮换审批流程与审计平台。
- membership 协议模型扩展（仍使用现有 `signature` 字符串字段）。
- 对 `signature_key_id` 治理语义重写。

## 接口 / 数据

### 新增策略校验入口（crate 内部）
```rust
validate_membership_snapshot_restore_policy(
    policy: &MembershipSnapshotRestorePolicy,
) -> Result<(), WorldError>

validate_membership_revocation_sync_policy(
    policy: &MembershipRevocationSyncPolicy,
) -> Result<(), WorldError>
```

### signer 公钥比较语义
- 当策略白名单非空时：
  - 签名必须是 `ed25519:v1:<public_key_hex>:<signature_hex>`。
  - `<public_key_hex>` 按 32-byte hex 解析并规范化为小写 hex。
  - 与策略白名单规范化集合比较（大小写无关）。

## 里程碑
- **HP6-M0**：设计文档 + 项目管理文档。
- **HP6-M1**：策略校验与规范化实现并接线。
- **HP6-M2**：测试回归、文档状态与 devlog 收口。

## 风险
- 严格校验会把历史脏配置显式暴露为错误，升级初期可能触发“从隐式容忍到显式拒绝”的运维告警。
- 策略白名单较大时每次同步/恢复的规范化成本上升，需要保持实现轻量（集合构建与去重一次完成）。
- 错误信息需保持稳定可读，避免排障成本上升。
