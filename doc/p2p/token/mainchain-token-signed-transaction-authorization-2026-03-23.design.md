# oasis7 主链 Token 签名交易鉴权（设计文档）

- 对应需求文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
- 对应项目管理文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.project.md`

审计轮次: 2
## 设计目标
- 在不重做整个资产动作协议的前提下，先关闭当前公开 `transfer submit` 面的未签名提交漏洞。
- 复用现有 `ed25519` 原语与 `awt:pk:<public_key_hex>` 账户派生规则，把请求级鉴权前置到 HTTP submit 入口。
- 把 signed transaction model 上提到 shared `ConsensusActionPayloadEnvelope` / `NodeRuntime` 提交层，避免未来新 submit surface 再次绕过。
- 明确这是统一 signed transaction model 的推进切片，而不是治理控制终态完成。

## 请求契约
| 字段 | 含义 | 规则 |
| --- | --- | --- |
| `from_account_id` | 主链转出账户 | 必须等于 `awt:pk:<normalized_public_key_hex>` |
| `to_account_id` | 主链转入账户 | 沿用现有账户格式规则 |
| `amount` | 转账数量 | `> 0` |
| `nonce` | 转账 nonce | `> 0`，通过现有 runtime 规则做 anti-replay |
| `public_key` | `ed25519` 公钥 hex | 32-byte，规范化为小写 |
| `signature` | 签名 hex | transfer 当前沿用固定域前缀 + 64-byte `ed25519` 签名 |

## Shared Payload Auth Envelope
| 字段 | 说明 |
| --- | --- |
| `auth.type` | 当前固定为 `main_token_action` |
| `auth.data.account_id` | 资产动作声明的账户/控制者标识 |
| `auth.data.public_key` | `ed25519` 公钥 |
| `auth.data.signature` | 固定域前缀签名 |

- `ConsensusActionPayloadEnvelope` 对非主链 Token action 继续允许 `auth=None`。
- 对 `TransferMainToken / ClaimMainTokenVesting / InitializeMainTokenGenesis / DistributeMainTokenTreasury`，`NodeRuntime` 必须要求 `auth=Some(main_token_action)`。

## Canonical Payload（shared）
- 每个主链 Token action 都使用同一签名外框：

```json
{
  "version": 1,
  "operation": "<action_operation>",
  "account_id": "<authorized_account_or_controller>",
  "public_key": "<public_key_hex>",
  "action": { "...runtime action json..." }
}
```

- `operation` 与签名前缀按 action 区分，用于域隔离。
- transfer HTTP 入口继续沿用请求级校验，再把已有签名材料写入 shared payload auth envelope。

## 提交层校验规则
| action | 提交层规则 | 当前安全结论 |
| --- | --- | --- |
| `TransferMainToken` | `auth.account_id == from_account_id` 且必须等于 `awt:pk:<public_key_hex>` | 账户绑定成立 |
| `ClaimMainTokenVesting` | `auth.account_id == beneficiary`；若 beneficiary 为 `awt:pk:`，需校验公钥派生；若为 `protocol:*` 等命名账户，只要求签名与 account_id 一致 | 已签名化，但命名控制账户的真实 controller binding 仍待治理专题 |
| `InitializeMainTokenGenesis` | 必须带 signed controller metadata；当前只保证 action + controller account_id 被签名 | 已签名化，但真实创世 signer allowlist / ceremony 仍待治理专题 |
| `DistributeMainTokenTreasury` | 必须带 signed controller metadata；当前只保证 action + controller account_id 被签名 | 已签名化，但真实 treasury governance signer 仍待治理专题 |

## 首切片边界
| 资产动作 | 当前状态 | 原因 |
| --- | --- | --- |
| `TransferMainToken` 公开 HTTP submit | `implemented` | 当前唯一公开资产提交面，已完成请求级签名鉴权 |
| Shared payload auth envelope | `implemented_in_this_slice` | 是所有未来 submit surface 的汇合点 |
| `ClaimMainTokenVesting` payload submit gate | `implemented_in_this_slice` | 已有 beneficiary，可先纳入 shared envelope |
| `InitializeMainTokenGenesis` payload submit gate | `implemented_in_this_slice` | 先要求 signed controller metadata，再留待治理绑定 |
| `DistributeMainTokenTreasury` payload submit gate | `implemented_in_this_slice` | 先要求 signed controller metadata，再留待治理绑定 |
| Governance signer allowlist / ceremony | `pending` | 需要 producer/QA/治理专题联审 |

## 错误码约定
| error_code | 触发条件 |
| --- | --- |
| `invalid_request` | JSON 缺字段、字段为空、公钥格式非法、金额/nonce 非法 |
| `invalid_signature` | 签名前缀不对、签名长度非法、签名验签失败 |
| `account_auth_mismatch` | `from_account_id` 不是该公钥派生账户 |
| `missing_main_token_auth` | token runtime action 在 payload 层缺 auth proof |
| `insufficient_balance` | 通过鉴权后余额不足 |
| `nonce_replay` | 通过鉴权后 nonce 不满足递增规则 |

## 兼容性与后续
- `oasis7_web_launcher` 只负责透传 transfer 新字段，不在本切片生成签名。
- Web/native 转账 UI 后续需要补签名材料采集与本地 signer 路径，否则提交会被后端拒绝。
- `genesis/treasury` 虽然进入 shared envelope，但 controller allowlist、真实 signer slot 绑定、external signer 与 ceremony 仍需后续专题完成。
