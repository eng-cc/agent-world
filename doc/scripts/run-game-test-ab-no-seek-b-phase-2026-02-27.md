# run-game-test-ab 无 Seek 的 B 段量化口径（2026-02-27）

## 目标
- 将 `scripts/run-game-test-ab.sh` 的 B 段从 `step/seek` 调整为不依赖 `seek` 的控制探针。
- 与“viewer live 模式不可回退”约束对齐，避免测试脚本继续使用禁用动作。

## 范围

### In Scope
- 更新 `scripts/run-game-test-ab.sh`：
  - A 段保持 `play/pause`。
  - B 段改为纯 `step` 探针（多次 step 推进验证）+ 无效动作探针。
  - 更新脚本帮助文案、`ab_metrics.md` 与 `card_quant_metrics.md` 的 B 段标签。
- 保持产物结构与 JSON 主体结构稳定（`ab_metrics.json/md`、`card_quant_metrics.md`）。

### Out of Scope
- 不改 `doc/game-test.md`（用户锁定文档）。
- 不追溯修改历史卡片与历史产物。
- 不改 `run-game-test.sh` 启停链路。

## 接口/数据
- B 段动作集合：`step(count=8)` + `step(count=2)`（期望推进）+ `move`（期望拒绝）。
- B 段判定：两个 `step` 均推进即 PASS。
- 摘要文本：`B (step chain)` / `B（step链路）`。

## 里程碑
1. M1：建档并确认无 seek 口径。
2. M2：脚本改造完成并通过脚本级验证。
3. M3：项目文档与 devlog 收口。

## 风险
- 历史依赖 `phaseB.seek` 字段的离线分析脚本可能受影响。
- 与旧卡片模板中的 `B（step/seek）` 文案暂时并存，短期存在口径混读风险。
