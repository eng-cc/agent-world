# Non-Viewer 链上鉴权协议重构（生产级）

## 目标
- 将 Viewer Live 控制链路从“`player_id + public_key` 字符串绑定”升级为“可验签、可防重放、可持久化”的生产级链上鉴权协议。
- 为 `prompt_control` 与 `agent_chat` 提供统一鉴权凭证（proof）格式，确保请求来源可验证、内容不可篡改、旧签名不可重放。
- 在不改动 viewer 视觉层的前提下，完成 non-viewer 生产级安全闭环。

## 范围

### In Scope
- `crates/agent_world_proto`
  - 新增玩家鉴权 proof 数据结构（ed25519 + nonce + signature）。
  - 扩展 `PromptControlApplyRequest` / `PromptControlRollbackRequest` / `AgentChatRequest`，支持携带鉴权 proof。
- `crates/agent_world`
  - 新增 Viewer 鉴权签名负载规范与验签实现（canonical payload）。
  - live 控制链路强制校验 proof。
  - 增加重放防护状态（按 player 维护已接受最大 nonce）并纳入 snapshot 持久化。
  - 新增错误码：缺失 proof、签名非法、key 不匹配、nonce 重放等。
- `crates/agent_world_viewer`
  - 更新非视觉发送路径：按环境变量密钥生成 proof 并附加到请求中。
- 测试
  - 协议 roundtrip/兼容测试。
  - live 正向/反向鉴权测试（签名篡改、player/key 不一致、nonce 重放）。
  - persist roundtrip 测试（nonce 状态保留）。

### Out of Scope
- viewer UI 样式与交互改造。
- 共识层（node/consensus）协议替换或 PoS 机制改造。
- 链上经济规则与 gameplay 数值平衡重做。

## 接口 / 数据

### 1) 玩家鉴权 proof（协议层）
- 新增 `PlayerAuthProof`：
  - `scheme`：当前为 `ed25519`
  - `player_id`
  - `public_key`
  - `nonce`（`u64`，单调递增）
  - `signature`
- 请求扩展：
  - `PromptControlApplyRequest.auth: Option<PlayerAuthProof>`
  - `PromptControlRollbackRequest.auth: Option<PlayerAuthProof>`
  - `AgentChatRequest.auth: Option<PlayerAuthProof>`

### 2) 规范化签名负载（canonical payload）
- 每类请求定义稳定签名 payload，至少包含：
  - 协议版本
  - 动作类型（preview/apply/rollback/chat）
  - 关键业务字段（agent/message/patch/version 等）
  - `player_id` / `public_key` / `nonce`
- 使用稳定编码构建签名字节，确保跨实现一致。

### 3) 重放防护状态
- `WorldModel` 新增：
  - `player_auth_last_nonce: BTreeMap<String, u64>`
- 语义：
  - 同一 `player_id` 的新请求 `nonce` 必须严格大于已接受值。
  - 通过 snapshot 持久化，重启后防重放仍生效。

### 4) 服务端鉴权顺序
1. 校验 proof 存在与格式。
2. 校验 proof 与请求字段一致性（player/key）。
3. 验签 canonical payload。
4. 校验并消费 nonce（防重放）。
5. 执行业务授权（agent 绑定检查）与业务逻辑。

## 里程碑
- M0：建档（设计 + 项管）完成并冻结任务。
- M1：协议层新增 proof + 鉴权内核实现完成，单测通过。
- M2：live 链路强制鉴权 + nonce 防重放 + persist 完成。
- M3：viewer 发送链路签名接入、回归测试通过、文档/devlog 收口。

## 风险
- 兼容性风险：旧客户端未附带 proof 会被拒绝。
  - 缓解：提供清晰错误码与 viewer 端签名接入。
- nonce 管理风险：客户端 nonce 回退导致请求被拒绝。
  - 缓解：客户端使用单调 nonce 生成策略；服务端返回明确 `auth_nonce_replay`。
- 签名负载漂移风险：请求字段变更导致验签不兼容。
  - 缓解：签名 payload 版本化（v1），并补充固定向量测试。

## 当前状态
- 状态：已完成（2026-02-22）
- 已完成：M0、M1、M2、M3
- 收口说明：non-viewer 链路已实现签名鉴权、防重放、持久化与客户端签名接入，定向回归通过。
