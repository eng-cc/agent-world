# 客户端启动器反馈窗口化（2026-03-02）

审计轮次: 5
- 对应项目管理文档: doc/world-simulator/launcher/game-client-launcher-feedback-window-2026-03-02.project.md

## 1. Executive Summary
- 将启动器内嵌的反馈表单改为“按钮入口 + 弹窗窗口”交互。
- 与“设置”入口保持一致的操作模式，降低主面板信息密度。
- 保持反馈提交能力不退化（分布式优先，失败回落本地）。

## 2. User Experience & Functionality
### In Scope
- `crates/agent_world_client_launcher/src/main.rs`
  - 新增“反馈 / Feedback”按钮。
  - 点击后打开反馈窗口，窗口内承载原有反馈表单（类型、标题、描述、反馈目录、提交按钮、校验提示、提交结果）。
  - 主面板移除原内嵌反馈区域。
- 适配必要单测与回归测试，确保编译与功能不回退。

### Out of Scope
- 不改造 `feedback_entry.rs` 的提交协议与数据结构。
- 不新增反馈历史查询或附件上传界面。
- 不改动链路端接口行为。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- 新 UI 入口：
  - 按钮：`反馈 / Feedback`
  - 行为：点击后打开反馈窗口。
- 反馈窗口字段保持与原行为一致：
  - `kind/title/description/output_dir`
- 提交流程保持：
  - `submit_feedback_with_fallback`（远端优先，本地回落）。

## 5. Risks & Roadmap
- M1（T0）：设计文档 + 项目管理文档。
- M2（T1）：完成窗口化改造与回归测试。
- M3（T2）：文档完成态、devlog、结项提交。

### Technical Risks
- 风险：窗口与主界面状态同步不一致导致反馈结果不可见。
  - 缓解：反馈状态统一保存在 `ClientLauncherApp`，窗口仅负责渲染。
- 风险：窗口关闭后用户误以为输入丢失。
  - 缓解：草稿状态驻留内存，除非手动修改/重启应用不会丢失。
- 风险：UI 拆分后引入 borrow 冲突导致编译失败。
  - 缓解：将窗口渲染逻辑收敛为独立方法，先构建文案副本再渲染。

## 完成态（2026-03-02）
- 启动器主面板已移除内嵌反馈表单，操作区新增“反馈 / Feedback”按钮。
- 点击“反馈 / Feedback”后会打开独立反馈窗口，窗口内保留原有能力：
  - 类型、标题、描述、反馈目录编辑
  - 必填校验提示
  - 提交结果提示
  - `submit_feedback_with_fallback` 提交流程（远端优先，失败回落本地）不变
- 回归验证通过：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_client_launcher`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_client_launcher`

## 6. Validation & Decision Record
- 追溯: 对应同名 `.project.md`，保持原文约束语义不变。
