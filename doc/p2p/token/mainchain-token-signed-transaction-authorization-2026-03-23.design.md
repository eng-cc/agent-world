# oasis7 主链 Token 签名交易鉴权（设计文档）

- 对应需求文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
- 对应项目管理文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.project.md`

审计轮次: 1
## 设计目标
- 在不重做整个资产动作协议的前提下，先关闭当前公开 `transfer submit` 面的未签名提交漏洞。
- 复用现有 `ed25519` 原语与 `awt:pk:<public_key_hex>` 账户派生规则，把请求级鉴权前置到 HTTP submit 入口。
- 明确这是统一 signed transaction model 的第一切片，而不是终态完成。

## 请求契约
| 字段 | 含义 | 规则 |
| --- | --- | --- |
| `from_account_id` | 主链转出账户 | 必须等于 `awt:pk:<normalized_public_key_hex>` |
| `to_account_id` | 主链转入账户 | 沿用现有账户格式规则 |
| `amount` | 转账数量 | `> 0` |
| `nonce` | 转账 nonce | `> 0`，通过现有 runtime 规则做 anti-replay |
| `public_key` | `ed25519` 公钥 hex | 32-byte，规范化为小写 |
| `signature` | 签名 hex | 固定前缀 + 64-byte `ed25519` 签名 |

## Canonical Payload
- 签名域前缀：`awttransferauth:v1:`
- canonical JSON 结构：

```json
{
  "version": 1,
  "payload": {
    "operation": "transfer_main_token_submit",
    "from_account_id": "awt:pk:<public_key_hex>",
    "to_account_id": "<target_account>",
    "amount": 7,
    "nonce": 2,
    "public_key": "<public_key_hex>"
  }
}
```

- 最终验签原文：`b"awttransferauth:v1:" + serde_json::to_vec(canonical_envelope)`

## 验证流水线
1. 解析 HTTP body，读取 `ChainTransferSubmitRequest`。
2. 规范化 `from_account_id/to_account_id/public_key/signature`。
3. 生成 `expected_from_account_id = awt:pk:<normalized_public_key_hex>`。
4. 若 `from_account_id != expected_from_account_id`，立即返回 `account_auth_mismatch`。
5. 对 canonical payload 执行 `ed25519` 验签；失败返回 `invalid_signature`。
6. 仅在签名通过后，继续执行现有余额/nonce 预检。
7. 预检通过后，构造原有 `Action::TransferMainToken` 并提交 consensus payload。

## 首切片边界
| 资产动作 | 当前状态 | 原因 |
| --- | --- | --- |
| `TransferMainToken` 公开 HTTP submit | `implemented_in_this_slice` | 当前唯一公开资产提交面，优先级最高 |
| `ClaimMainTokenVesting` | `pending` | 尚未发现同等级公开 submit 面，本轮只冻结协议要求 |
| `InitializeMainTokenGenesis` | `pending` | 需与创世 ceremony / signer policy 一起设计 |
| `DistributeMainTokenTreasury` | `pending` | 需与治理 signer / treasury policy 一起设计 |

## 错误码约定
| error_code | 触发条件 |
| --- | --- |
| `invalid_request` | JSON 缺字段、字段为空、公钥格式非法、金额/nonce 非法 |
| `invalid_signature` | 签名前缀不对、签名长度非法、签名验签失败 |
| `account_auth_mismatch` | `from_account_id` 不是该公钥派生账户 |
| `insufficient_balance` | 通过鉴权后余额不足 |
| `nonce_replay` | 通过鉴权后 nonce 不满足递增规则 |

## 兼容性与后续
- `oasis7_web_launcher` 只负责透传新字段，不在本切片生成签名。
- Web/native 转账 UI 后续需要补签名材料采集与本地 signer 路径，否则提交会被后端拒绝。
- 若后续要把 signed transaction model 上提到 consensus action envelope，需要新专题统一 `action_type / signer_set / nonce_scope / domain separation`，本文件不提前定义。
