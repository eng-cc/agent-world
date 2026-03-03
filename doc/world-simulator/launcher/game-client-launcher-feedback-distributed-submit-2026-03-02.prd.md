# 客户端启动器反馈分布式提交迁移（2026-03-02）

## 1. Executive Summary
- 将“给团队提 Bug/建议”能力从“仅本地 JSON 落盘”升级为“启动器优先提交到分布式反馈网络”。
- 复用已有 `agent_world_node` + `agent_world_distfs` 反馈链路，确保反馈进入 `feedback` append-only 存储并触发 P2P 广播。
- 保持用户可用性：当链运行时不可用或提交失败时，自动回落到本地反馈包落盘，不丢反馈。

## 2. User Experience & Functionality
### In Scope
- `crates/agent_world/src/bin/world_chain_runtime.rs`
  - 启用 `feedback_p2p` 配置（默认参数）。
  - 新增 `POST /v1/chain/feedback/submit` 接口。
  - 接口接收启动器反馈载荷，服务端构造并签名 `FeedbackCreateRequest`，调用 `NodeRuntime::submit_feedback`。
- `crates/agent_world_client_launcher`
  - 反馈提交流程改为“远端分布式提交优先 + 本地落盘回落”。
  - UI 提示区分“已提交到分布式网络”与“已本地保存（回落）”。
- 新增/更新单元测试，覆盖请求解析、提交成功与回落路径。

### Out of Scope
- 不新增反馈浏览/检索 UI。
- 不在本期扩展 append/tombstone 前端入口。
- 不引入外部数据库或中心化反馈服务。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
### 新增 HTTP 接口
- `POST /v1/chain/feedback/submit`
- 请求 JSON：
  - `category`: `"bug" | "suggestion"`
  - `title`: `string`
  - `description`: `string`
  - `platform`: `string`（可选，默认 `client_launcher`）
  - `game_version`: `string`（可选，默认 `unknown`）
- 响应 JSON（成功）：
  - `ok: true`
  - `feedback_id`, `event_id`, `audit_id`, `created_at_ms`
- 响应 JSON（失败）：
  - `ok: false`
  - `error: string`

### 启动器提交策略
- 若 `chain_enabled=true`：先向 `chain_status_bind` 提交 `POST /v1/chain/feedback/submit`。
- 若远端失败：记录失败原因，自动回落 `submit_feedback_report` 本地落盘。
- 若 `chain_enabled=false`：直接本地落盘。

## 5. Risks & Roadmap
- M1（T0）：设计文档 + 项目管理文档建档。
- M2（T1）：链运行时反馈提交接口实现完成。
- M3（T2）：启动器远端优先提交 + 本地回落实现完成。
- M4（T3）：测试、文档与 devlog 收口。

### Technical Risks
- 风险：远端反馈接口未启动或 `feedback_p2p` 未启用导致提交失败。
  - 缓解：启动器自动回落本地落盘，并明确展示回落原因。
- 风险：反馈内容超出 DistFS `max_content_bytes` 限制。
  - 缓解：服务端构造提交内容时做长度收敛并返回可读错误。
- 风险：启动器与链运行时 JSON 协议不一致。
  - 缓解：两端新增对应单元测试并固定字段名。

## 完成态（2026-03-02）
- `world_chain_runtime` 已默认启用 `feedback_p2p`，并新增 `POST /v1/chain/feedback/submit`。
- 新接口会在服务端完成请求校验、构造并签名 `FeedbackCreateRequest`，然后调用 `NodeRuntime::submit_feedback`，反馈进入 DistFS 并触发 P2P announce 流程。
- 启动器反馈提交已升级为“分布式提交优先 + 本地落盘回落”：
  - 分布式成功：UI 显示“已提交到分布式网络”并展示 `feedback_id/event_id`。
  - 分布式失败：自动本地保存 JSON，并在日志中记录回落原因。
- 相关单元测试已补齐并通过，`main.rs` 与 `world_chain_runtime.rs` 均维持在 1200 行以内。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
