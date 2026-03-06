# ROUND-003 文件命名语义审查执行清单

## 目标
- 形成“命名语义审读 -> 更名决策 -> 索引/引用回写 -> 审计轮次更新”的可执行闭环。
- 提升 PRD/Project/Index 文件名可读性与可追溯性，降低误导风险。

## 执行阶段
| 阶段 | 动作 | 产物 | 状态 |
| --- | --- | --- | --- |
| P0 | 建立启动文档与审读清单 | `consistency-review-round-003.md`、`round-003-reviewed-files.md`、本清单 | done |
| P1 | 盘点命名问题（A~E 分区） | 命名问题清单（I3-*） | done |
| P2 | 输出更名方案与索引/引用回写 | 更名方案表 + 引用回写清单 | done |
| P3 | 执行更名与引用替换 | 文档重命名 + `prd.index.md`/引用更新 | done |
| P4 | 回写 `审计轮次: 3` 与复审结论 | `S_round003` + ROUND-003 复审结果 | done |

注：`审计轮次: 3` 已回写，更名执行与复审结论已收口。

## 命名语义检查清单（摘要）
- 文件名能表达模块/主题/范围（必要时含阶段/日期）。
- 避免 `misc/tmp/update/new/fix` 等非语义化词作为主干。
- `phase/round` 仅在确有阶段序列时使用；无后续阶段需合并或更名。
- `*.prd.md` 与 `*.prd.project.md` 必须同名成对。
- 入口类文件（`prd.md`/`prd.project.md`/`prd.index.md`）固定命名不参与更名。

## 更名执行规则
- 一次更名必须同时更新：
  - 模块 `prd.index.md` 索引入口。
  - 所有引用路径（PRD/project/README/站点/脚本）。
- 若更名被延后，需在 ROUND-003 “命名问题清单/整改项”登记原因与计划。

## 通用验收命令
- `test -f doc/core/reviews/consistency-review-round-003.md`
- `test -f doc/core/reviews/round-003-reviewed-files.md`
- `test -f doc/core/reviews/round-003-filename-semantic-worklist.md`
- `rg -n "I3-|A3-" doc/core/reviews/consistency-review-round-003.md`
- `./scripts/doc-governance-check.sh`
