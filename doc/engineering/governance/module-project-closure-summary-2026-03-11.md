# 全仓模块主项目收口总览（2026-03-11）

审计轮次: 4

## 目标
- 汇总 2026-03-11 本轮连续收口后的模块主项目状态，为下一轮 PRD / project 拆解提供干净起点。
- 明确当前仓内 `doc/*/project.md` 已无未完成主项目项，后续工作应按“新需求 -> 新任务”进入，而不是沿用旧尾注。

## 模块状态总览
| 模块 | 主项目状态 | 下一任务 | 备注 |
| --- | --- | --- | --- |
| `core` | `completed` | `无` | 一致性审查链路已收口 |
| `engineering` | `completed` | `无` | 迁移专项、趋势基线、季度治理模板均已落档 |
| `game` | `completed` | `无` | `TASK-GAME-018` 证据回填尾注已收口 |
| `headless-runtime` | `completed` | `无` | 模块状态回写已对齐 |
| `p2p` | `completed` | `无` | 等待新需求 |
| `playability_test_result` | `completed` | `无` | 模块状态回写已对齐 |
| `readme` | `completed` | `无` | 清单 / 链接检查 / 季度节奏已收口 |
| `scripts` | `completed` | `无` | 分层 / 契约 / 趋势已收口 |
| `site` | `completed` | `无` | CTA 优先级调整已收口 |
| `testing` | `completed` | `无` | 趋势基线已收口 |
| `world-runtime` | `completed` | `无` | 模块主项目无未完成项 |
| `world-simulator` | `completed` | `无` | 模块状态回写已对齐 |

## 本轮关键动作
- 收口 `readme`：季度口径审查与修复节奏建档。
- 收口 `engineering`：迁移复核、趋势统计、季度治理模板、umbrella 迁移任务全部关闭。
- 收口 `game`：`TASK-GAME-018` 对 playability / testing / core 的证据回填尾注关闭。
- 修正模块状态漂移：`playability_test_result`、`headless-runtime`、`world-simulator` 主项目状态统一回写为 `completed`。

## 下一轮入口建议
- 只有在出现新需求、新 PRD-ID 或新增阶段目标时，才重新打开模块主项目。
- 若进入新一轮规划，建议先从 `producer_system_designer` 视角新增优先级清单，再按 owner role 拆到对应模块 `prd.md` / `project.md`。
- 若只是例行治理延续，优先复用现有季度模板与趋势基线，而不是重建流程文档。

## 验证命令
- `for f in doc/*/project.md; do rg -n "^- 当前状态:|^- 下一任务:" "$f"; done`
- `git log --oneline -12`
