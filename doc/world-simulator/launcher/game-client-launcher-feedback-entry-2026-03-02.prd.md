# 客户端启动器反馈入口（2026-03-02）

- 对应项目管理文档: doc/world-simulator/launcher/game-client-launcher-feedback-entry-2026-03-02.prd.project.md

## 1. Executive Summary
- 在桌面客户端启动器中提供面向玩家的“提交反馈”入口，覆盖 `Bug` 与`建议`两类反馈。
- 反馈提交时自动附带最小可复现上下文（启动器配置快照 + 最近日志），降低研发排查成本。
- 首版不依赖远端服务，先落地本地可追踪反馈包（JSON 文件）以形成可用闭环。

## 2. User Experience & Functionality
### In Scope
- 修改 `crates/agent_world_client_launcher`：新增反馈输入区（类型、标题、描述、提交按钮）。
- 提交行为将反馈写入本地目录（默认 `feedback/`），文件名携带时间戳与类别。
- 自动附带数据：
  - 启动器当前配置快照（scenario/bind/LLM/chain 等）
  - 最近日志窗口（截断到固定上限）
  - 生成时间（RFC3339）
- 补充单元测试，覆盖反馈记录序列化与文件名/时间戳安全性。

### Out of Scope
- 不接入远端反馈 API（HTTP 上报/鉴权/重试）。
- 不实现附件上传（截图、崩溃 dump）。
- 不改动 `world_game_launcher` 或 `world_viewer_live` 协议。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
### UI 字段
- 反馈类型：`Bug` / `Suggestion`
- 标题：单行文本，必填
- 描述：多行文本，必填
- 反馈目录：默认 `feedback`，可在 UI 中编辑，目录不存在时自动创建

### 落盘结构（JSON）
- `kind`: `"bug" | "suggestion"`
- `title`: `string`
- `description`: `string`
- `created_at`: `string`（RFC3339）
- `launcher_config`: 对应 `LaunchConfig` 快照
- `recent_logs`: `string[]`（最新 N 行）

### 文件命名
- `{YYYYMMDDTHHMMSSZ}-{kind}.json`
- 示例：`20260302T070501Z-bug.json`

## 5. Risks & Roadmap
- M1（T0）：设计/项目文档建档，任务拆解完成。
- M2（T1）：启动器反馈入口与本地反馈包落盘实现完成，单元测试通过。
- M3（T2）：回归测试、文档与 devlog 收口，项目结项。

### Technical Risks
- 风险：本地反馈目录权限不足导致写入失败。
  - 缓解：UI 显示明确失败原因，允许用户修改反馈目录后重试。
- 风险：日志过大导致反馈文件膨胀。
  - 缓解：仅保留最近固定行数（如 200 行）。
- 风险：主文件继续膨胀接近 1200 行限制。
  - 缓解：本次同步拆分反馈模块到独立文件，维持单文件行数约束。

## 完成态（2026-03-02）
- 桌面启动器已提供“反馈（Bug / 建议）”入口，支持类型、标题、描述与反馈目录输入。
- 提交反馈后会自动生成本地 JSON 反馈包，包含：
  - 反馈内容（类型/标题/描述）
  - 生成时间（UTC）
  - 启动器配置快照
  - 最近 200 行日志
- 反馈写入失败时可在 UI 与日志中看到明确错误提示，便于用户调整目录后重试。
- `main.rs` 测试已拆分到 `main_tests.rs`，并新增 `feedback_entry.rs` 模块，满足单文件行数约束。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
