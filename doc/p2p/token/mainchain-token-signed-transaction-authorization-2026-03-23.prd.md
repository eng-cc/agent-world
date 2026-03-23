# oasis7 主链 Token 签名交易鉴权（2026-03-23）

- 对应设计文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.design.md`
- 对应项目管理文档: `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.project.md`

审计轮次: 1
## 1. Executive Summary
- Problem Statement: 当前 `oasis7_chain_runtime` 的 `POST /v1/chain/transfer/submit` 仍接受未签名 JSON，请求中的 `from_account_id` 没有和真实签名人绑定，导致公开资产入口仍处于 preview-grade 而不是主流公链常见的“签名授权后再执行”模型。
- Proposed Solution: 为主链 Token 资产动作建立统一签名交易鉴权专题，首个实现切片先收口 `TransferMainToken` 的公开提交入口，要求请求携带 `public_key + signature`，并把 `from_account_id` 绑定为 `awt:pk:<public_key_hex>`。
- Success Criteria:
  - SC-1: `/v1/chain/transfer/submit` 不再接受未签名请求；缺少 `public_key` 或 `signature` 的请求必须被拒绝。
  - SC-2: 只有当 `from_account_id == awt:pk:<normalized_public_key_hex>` 且 `ed25519` 签名校验通过时，转账请求才允许继续进入余额/nonce 预检与共识提交。
  - SC-3: 鉴权失败需输出结构化错误，至少区分 `invalid_signature` 与 `account_auth_mismatch`。
  - SC-4: `oasis7_web_launcher` 控制面代理请求结构与 runtime 新鉴权字段保持一致，不出现字段脱节。
  - SC-5: 形成后续路线图，明确 `ClaimMainTokenVesting / InitializeMainTokenGenesis / DistributeMainTokenTreasury` 仍需并入统一 signed transaction 模型。

## 2. User Experience & Functionality
- User Personas:
  - `runtime_engineer`：需要先关闭当前最暴露的公开资产提交面。
  - `qa_engineer`：需要把签名缺失、签名错误、账户不匹配变成可验证的阻断用例。
  - `producer_system_designer`：需要看到“P0 已开始收口，但仍未完成所有资产动作”的真实阶段。
  - `viewer_engineer`：需要知道 Web/native 转账入口后续必须提供签名材料，而不是再提交裸 JSON。
- User Scenarios & Frequency:
  - 链上转账提交：每次玩家或运营侧通过公开 runtime/control-plane 入口提交主链 Token 转账时触发。
  - 安全回归：每次修改资产提交接口或转账交互时触发。
  - 创世前安全推进：每次 producer 复核 `not_mainnet_grade` blocker 是否有实质收口时触发。
- User Stories:
  - PRD-P2P-TXAUTH-001: As a `runtime_engineer`, I want transfer submit to require a valid signature, so that公开资产入口不再信任裸 JSON 的 `from_account_id`。
  - PRD-P2P-TXAUTH-002: As a `qa_engineer`, I want invalid/mismatched auth to fail with stable error codes, so that release gate can block unsigned asset surfaces.
  - PRD-P2P-TXAUTH-003: As a `producer_system_designer`, I want this专题明确写出“首切片只完成 transfer submit”， so that阶段判断不会把 P0 的开始误判成全部资产动作已收口。
- Critical User Flows:
  1. Flow-P2P-TXAUTH-001: `客户端构造 transfer request -> 使用 ed25519 对 canonical payload 签名 -> 提交 public_key/signature -> runtime 验证 -> 通过后继续余额/nonce 预检 -> 进入 consensus payload`
  2. Flow-P2P-TXAUTH-002: `请求缺少签名或签名格式非法 -> runtime 直接返回 invalid_request/invalid_signature -> 不进入余额检查与提交流程`
  3. Flow-P2P-TXAUTH-003: `public_key 签名有效但 from_account_id 不等于 awt:pk:<public_key_hex> -> runtime 返回 account_auth_mismatch -> 不允许冒用其他账户`
  4. Flow-P2P-TXAUTH-004: `producer 复核专题状态 -> 看到 transfer submit 已签名化 -> 仍保留 claim/genesis/treasury 为后续任务 -> 继续维持 not_mainnet_grade 口径`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算/判定规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Transfer submit 鉴权请求 | `from_account_id/to_account_id/amount/nonce/public_key/signature` | runtime 解析 JSON 并校验字段完整性 | `raw_request -> parsed_request` | `amount > 0`、`nonce > 0`、字段不可空、账户 ID 仍遵守现有格式约束 | 任何公开调用方都必须显式提供签名材料 |
| Canonical signing payload | `version/operation/from_account_id/to_account_id/amount/nonce/public_key` | 以固定字段顺序编码为 canonical JSON，并加固定域前缀后验签 | `parsed_request -> auth_verified/auth_rejected` | 签名域固定为 transfer submit，避免与 viewer/chat auth 混用 | 仅持有对应私钥的请求方可生成有效签名 |
| 账户所有权绑定 | `derived_from_account_id = awt:pk:<public_key_hex>` | runtime 比对 `request.from_account_id` 与派生账户 | `auth_verified -> bound/rejected` | 若不相等则返回 `account_auth_mismatch` | 请求人只能提交自己公钥派生的主链账户 |
| 预检与提交流程 | `balance/last_nonce/action_id/payload` | 鉴权通过后复用既有余额/nonce 预检与 consensus submit | `bound -> accepted/rejected` | 先 auth，后 balance/nonce；失败时不写入 accepted tracker | 资产提交权限以签名通过为前置 |
| 后续资产动作扩展位 | `action_surface/status/next_owner` | 记录 `claim/genesis/treasury` 仍待接入统一模型 | `draft -> planned -> implemented` | 只有 transfer submit 在本切片实现，其他动作保持 pending | 跨角色协作后再扩展，不在本轮偷渡完成 |
- Acceptance Criteria:
  - AC-1: `oasis7_chain_runtime` 的 `ChainTransferSubmitRequest` 必须新增 `public_key` 与 `signature` 字段，并在缺失时拒绝请求。
  - AC-2: runtime 必须验证 `ed25519` 签名，且 `from_account_id` 必须严格等于 `awt:pk:<normalized_public_key_hex>`。
  - AC-3: 签名无效、签名格式非法、公钥格式非法、账户绑定不匹配必须返回结构化错误，不得继续进入 `preflight_validate_transfer_request`。
  - AC-4: 有效签名请求仍必须保留现有 `same account / amount / nonce / insufficient balance / nonce replay` 行为和错误语义。
  - AC-5: `oasis7_web_launcher` 的控制面请求结构、序列化与代理测试必须同步更新到新字段集合。
  - AC-6: 本专题必须明确声明本轮仅收口 `TransferMainToken` 公开提交面，`ClaimMainTokenVesting / InitializeMainTokenGenesis / DistributeMainTokenTreasury` 仍在待办链。
  - AC-7: 定向 required 回归必须覆盖有效签名成功、缺签名拒绝、错误签名拒绝、`from_account_id` 与 `public_key` 不匹配拒绝。
  - AC-8: 专题文档必须接入 `doc/p2p/prd.md`、`doc/p2p/project.md`、`doc/p2p/prd.index.md` 与 `doc/p2p/README.md`。
- Non-Goals:
  - 本轮不实现生产级 keystore、HSM/KMS、硬件钱包或外部 signer 服务。
  - 本轮不把 `ClaimMainTokenVesting / InitializeMainTokenGenesis / DistributeMainTokenTreasury` 一次性全部接入。
  - 本轮不重做 Web/native 转账 UI 交互与助记词/钱包管理体验。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 客户端或控制面先构造主链转账 canonical payload，再以 `ed25519` 对该 payload 签名。`oasis7_chain_runtime` 在 HTTP 入口先解析并校验 `public_key/signature`，再执行账户绑定校验，最后才复用既有余额/nonce 预检与 consensus action payload 提交流程。
- Integration Points:
  - `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api.rs`
  - `crates/oasis7/src/bin/oasis7_chain_runtime/transfer_submit_api_tests.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher/control_plane.rs`
  - `crates/oasis7/src/runtime/main_token.rs`
  - `doc/p2p/blockchain/p2p-mainnet-crypto-security-baseline-2026-03-23.prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 若 `public_key` 不是 32-byte hex，直接按 `invalid_request` 拒绝。
  - 若 `signature` 不带预期前缀或不是 64-byte hex，按 `invalid_signature` 拒绝。
  - 若 `public_key` 大小写混用，允许通过规范化为小写 hex 后参与派生与验签。
  - 若签名通过但 `from_account_id` 不是该公钥派生账户，按 `account_auth_mismatch` 拒绝。
  - 若鉴权通过但余额不足或 nonce 回放，继续返回现有业务错误，不改变既有预检规则。
  - 若后续其他资产动作仍未签名化，本专题不得据此把整体 verdict 提升为 `mainnet_grade`。
- Non-Functional Requirements:
  - NFR-P2P-TXAUTH-1: 公开转账提交面不存在“缺签名也能继续执行余额/nonce 预检”的旁路。
  - NFR-P2P-TXAUTH-2: transfer auth 仅接受 `ed25519` 32-byte public key 与 64-byte signature，编码为 hex 且具备固定版本前缀。
  - NFR-P2P-TXAUTH-3: canonical payload 必须稳定、可回归，同一请求字段在相同顺序下得到完全一致的签名原文。
  - NFR-P2P-TXAUTH-4: 定向 required 回归命令必须在同一提交中给出，且 `git diff --check` 通过。
  - NFR-P2P-TXAUTH-5: 在 `claim/genesis/treasury` 未接入前，模块级安全 verdict 仍保持 `not_mainnet_grade`。
- Security & Privacy: 请求只上传公钥与签名，不上传私钥或助记词；runtime 只基于请求内容验签，不在本轮引入任何本地私钥托管逻辑。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 为 `POST /v1/chain/transfer/submit` 落地签名鉴权、账户绑定与 required 回归。
  - v1.1: 将 `ClaimMainTokenVesting / InitializeMainTokenGenesis / DistributeMainTokenTreasury` 接入同一 signed transaction envelope。
  - v2.0: 将签名交易模型上提到统一 consensus/asset action 层，并与 keystore / signer rotation 专题合流。
- Technical Risks:
  - 风险-1: Web/native 现有转账入口若未同步提供签名材料，会在后端收口后直接变成拒绝路径。
  - 风险-2: 若 canonical payload 定义不稳定，后续多端实现会出现签名不兼容。
  - 风险-3: 若 producer 误把“transfer submit 已签名化”解读成“全部资产动作已安全闭环”，会再次高估安全阶段。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-TXAUTH-001 | STRAUTH-0/1 | `test_tier_required` | 专题 PRD/design/project 建档、transfer submit 鉴权实现、request schema 更新 | 主链 Token 公开转账提交面 |
| PRD-P2P-TXAUTH-002 | STRAUTH-1/3 | `test_tier_required` | runtime/control-plane 定向测试，覆盖缺签名/错签名/账户不匹配 | 错误语义、阻断门禁与 QA 回归 |
| PRD-P2P-TXAUTH-003 | STRAUTH-2/3 | `test_tier_required` | project 路线图、devlog 与模块入口回写，确认 claim/genesis/treasury 仍 pending | producer 阶段判断与后续优先级 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-P2P-TXAUTH-001 | 先关闭公开 transfer submit 的无签名入口 | 一次性同时改 transfer/claim/genesis/treasury | 当前公开攻击面首先在 transfer submit；先关最暴露入口能最快实质降低风险。 |
| DEC-P2P-TXAUTH-002 | `from_account_id` 绑定为 `awt:pk:<public_key_hex>` | 继续接受任意字符串账户并只验“有签名即可” | 当前唯一已有明确公钥到账户派生语义的主链账户模型就是 `awt:pk:`。 |
| DEC-P2P-TXAUTH-003 | runtime 请求层先验签，再进入余额/nonce 预检 | 继续沿用“先业务预检，最后再验签” | 未通过签名的请求不应消耗任何资产语义校验路径。 |
| DEC-P2P-TXAUTH-004 | 本专题显式保留 `claim/genesis/treasury pending` | 把 transfer submit 收口后对外宣称 signed transaction model 已完成 | 安全基线专题已经把“所有资产动作统一签名模型”定义为 mainnet-ready blocker，不能偷换口径。 |
