# Agent World Runtime：零信任多节点治理与签名加固（2026-02-26）

## 目标
- 在“不可信多节点/对抗环境”前提下，补齐 runtime 在工件真实性、治理落地原子性、收据签名可信度、执行错误可观测性上的关键缺口。
- 保持现有 WASM ABI 与事件溯源框架可回放，不引入破坏性迁移。

## 范围

### In Scope
- P0 工件真实性链：
  - `artifact_identity` 从“可选”收敛为“必填”。
  - 禁止 `unsigned:*` 身份签名。
  - 注册（shadow/validate）与加载（load/persist）两条路径都执行工件签名验真与信任校验。
- P0 治理与共识绑定：
  - `apply_proposal` 需绑定共识最终性证明与多签门限校验。
  - `apply` 顺序改成原子提交语义：先验证与模块事件应用，最后写 `Governance::Applied` 与 `ManifestUpdated`。
- P1 收据签名升级：
  - 在现有 `HmacSha256` 之外增加节点签名与阈值签名算法。
  - 收据签名负载纳入 `consensus_height` 和 `receipts_root`，形成链上锚定。
- P1 错误可观测性：
  - Wasmtime 执行错误区分 `OutOfFuel` 与 `Interrupt`，不再统一折叠为 `Timeout`。

### Out of Scope
- 完整 PKI（证书吊销、在线信任服务、跨域 CA）。
- 共识模块内的拜占庭协议升级（仅消费最终性结果与签名材料）。
- 变更 WASM 模块业务语义。

## 接口 / 数据

### 1) 工件真实性链
- `ModuleArtifactIdentity`：
  - 新增 `signature_scheme`（例如 `ed25519`）与 `signer_node_id`。
  - `artifact_signature` 不再允许 `unsigned:*`。
- `World`：
  - 新增受信任签名人集合（按 `node_id -> public_key`）。
  - `validate_module_manifest` 和 `load_module_store_from_dir` 都调用统一验签函数。

### 2) 治理共识绑定与原子 apply
- 新增 `GovernanceFinalityCertificate`：
  - `world_id`、`proposal_id`、`manifest_hash`、`consensus_height`、`required_signers`、`signatures`。
- `apply_proposal` 新签名：
  - `apply_proposal(proposal_id, finality_certificate)`。
- 原子序：
  1. 校验 proposal 状态、manifest 与 certificate 一致性。
  2. 校验多签门限与签名有效性。
  3. 应用 module changes。
  4. 写 `ManifestUpdated`。
  5. 写 `Governance::Applied`（最后写入，避免“已应用但未完成”）。

### 3) 收据签名升级
- `SignatureAlgorithm` 新增：
  - `Ed25519`（节点单签）。
  - `ThresholdEd25519`（阈值多签，V1 用聚合签名声明 + 最小签名集合校验）。
- `ReceiptSignature` 新增字段：
  - `signer_node_id`（单签必填）。
  - `threshold`、`participants`（阈值签名场景）。
  - `consensus_height`、`receipts_root`（锚定字段）。
- `ReceiptSigner`：
  - 保留 `hmac_sha256` 以兼容历史测试。
  - 新增 `ed25519(...)` 与 `threshold_ed25519(...)`。

### 4) 执行错误可观测性
- `ModuleCallErrorCode` 新增：
  - `OutOfFuel`、`Interrupted`。
- Wasmtime trap 映射：
  - `Trap::OutOfFuel -> OutOfFuel`
  - `Trap::Interrupt -> Interrupted`

## 里程碑
- T0：设计建档与项目拆解。
- T1：P0 工件真实性链。
- T2：P0 治理共识绑定 + 原子 apply。
- T3：P1 收据签名升级与共识锚定。
- T4：P1 执行错误可观测性补强。
- T5：回归测试、文档回写与收口。

## 风险
- 历史工件若缺失合法签名会在加载阶段被拒绝，需要迁移脚本补签。
- 治理接口签名变化会影响现有调用方测试，需要一次性同步。
- 阈值签名在无完整聚合库时先走“参与者签名集合”校验，后续可替换为真实聚合签名。
