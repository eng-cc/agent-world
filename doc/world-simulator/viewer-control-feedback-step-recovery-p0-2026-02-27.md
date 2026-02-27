# Viewer 控制反馈 P0：Step 卡住恢复与强反馈（2026-02-27）

## 目标
- 修复 `step` 出现 `accepted` 但无推进（`tick/eventSeq` 不变）时的玩家失控感。
- 将“未推进原因 + 下一步建议”升级为玩家可见、可执行的强反馈，不依赖日志。
- 在不改 runtime 规则的前提下，优先保证 Web 闭环 A/B 测试中 B 段可恢复推进。

## 范围

### In Scope
- `web_test_api` 控制反馈闭环增强：
  - 识别 `step` 在 connected 下的无进展卡住。
  - 对 `step` 卡住自动发起一次 `seek(tick+1)` 恢复动作。
  - 反馈文案显式区分“未推进原因（Cause）”与“下一步建议（Next）”。
- 保持现有 Mission HUD 控制反馈卡片展示链路，直接消费增强后的 `reason/hint/effect`。

### Out of Scope
- Runtime/Live server 的核心步进算法重构。
- 非 Web Test API 控制源（普通按钮）的协议升级。
- 大规模 UI 结构改版。

## 接口/数据
- 现有 `WebTestApiControlFeedback` 继续作为单一反馈载体：
  - `stage`：`received/executing/applied/blocked`
  - `reason`：写入未推进原因（Cause）
  - `hint`：写入下一步建议（Next）
  - `effect`：写入自动恢复动作结果
- `publish_web_test_api_state` 引入 `ViewerClient`（可选）用于卡住时下发恢复控制。

## 里程碑
- M1：文档建档（T0）
- M2：`step` 卡住自动恢复 + 强反馈文案落地（T1）
- M3：测试与 A/B 实测闭环，文档收口（T2）

## 风险
- 风险：自动恢复可能掩盖底层真实阻塞。
  - 缓解：在 `reason/effect` 中明确“step 卡住已触发恢复”，保留可追踪语义。
- 风险：恢复动作选择不当导致跳跃体验。
  - 缓解：优先使用最小步长 `seek(tick+1)`，并提示可选 `play/seek+12`。
- 风险：文件行数接近上限导致维护成本上升。
  - 缓解：仅在 `web_test_api` 最小增量修改，避免触碰已达行数上限文件。

## 完成态（2026-02-27）
- 已落地：`step` 卡住超阈值后自动补发 `seek(tick+1)`，并在控制反馈中写入 `Cause/Next` 强反馈文案。
- A/B 实测：`output/playwright/playability/20260227-205043/` 中 `phase_b_step` 已推进（`tick 2 -> 3`）。
- 剩余：B 段仍受 `seek` 偶发无推进影响，后续可单独立项治理 `seek` 语义一致性。
