# Agent World Runtime：成员目录快照签名与来源校验（设计文档）

## 目标
- 为成员目录 DHT 快照增加签名能力，降低离线恢复时读取伪造快照的风险。
- 在恢复入口引入来源校验策略（trusted requester），避免任意节点覆盖本地 validator 目录。
- 保持与现有 P3.12 接口兼容：未启用策略时仍可按旧路径恢复。

## 范围

### In Scope（本次实现）
- 为成员目录广播/快照增加可选签名字段（向后兼容旧数据）。
- 新增 `MembershipDirectorySigner`（HMAC-SHA256）用于快照签名与验签。
- 新增恢复策略 `MembershipSnapshotRestorePolicy`：
  - `trusted_requesters` 来源白名单
  - `require_signature` 是否强制签名
- 新增 `restore_membership_from_dht_verified`：在恢复前执行 world/requester/signature 校验。
- 新增 `publish_membership_change_with_dht_signed`：广播与 DHT 同步写入带签名快照。

### Out of Scope（本次不做）
- 非对称密钥基础设施（PKI）与节点证书分发。
- 快照多版本链路、历史追踪与回滚策略。
- 跨世界全局授权中心与治理投票流程。

## 接口 / 数据

### 数据结构
- `MembershipDirectorySnapshot` 新增可选字段：
  - `signature: Option<String>`（hex）
- `MembershipDirectoryAnnounce` 新增可选字段：
  - `signature: Option<String>`

### 签名器
- `MembershipDirectorySigner::hmac_sha256(key)`
- `MembershipDirectorySigner::sign_snapshot(snapshot) -> signature_hex`
- `MembershipDirectorySigner::verify_snapshot(snapshot) -> Result<()>`

### 恢复策略
- `MembershipSnapshotRestorePolicy`
  - `trusted_requesters: Vec<String>`
  - `require_signature: bool`

### 同步客户端新增 API
- `publish_membership_change_with_dht_signed(...)`
- `restore_membership_from_dht_verified(...)`

## 里程碑
- **MA1**：补充签名字段并保持序列化兼容。
- **MA2**：实现快照签名器与签名发布链路。
- **MA3**：实现恢复阶段来源与签名校验策略。
- **MA4**：补充测试、更新项目文档与 devlog。

## 风险
- HMAC 方案依赖共享密钥管理，密钥泄露会影响信任边界。
- 仅在恢复入口做校验，实时广播路径默认仍可走兼容模式。
- `trusted_requesters` 为空时策略退化为宽松模式，部署时需显式配置。
