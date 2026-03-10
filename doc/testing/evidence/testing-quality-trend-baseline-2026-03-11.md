# 测试质量趋势基线（2026-03-11）

审计轮次: 4

## Meta
- Baseline ID: `TEST-TREND-BASELINE-20260311`
- 统计窗口: `2026-03-08` ~ `2026-03-11`
- 汇总角色: `qa_engineer`
- 样本数: `3`
- 指标结论: `yellow`

## 指标定义
- 首次通过率：`首次结论=pass` 的样本数 / 总样本数。
- 最终收口率：最终结论为 `pass/pass_after_fix/conditional_pass` 的样本数 / 总样本数。
- 阶段内逃逸率：首次不是 `pass`、且问题在下游 QA/可用性审查/复验阶段才暴露的样本数 / 总样本数。
- 平均修复时长：`关闭日期 - 发现日期` 的自然日均值；same-day 记为 `0d`。

## 红黄绿阈值
| 指标 | Green | Yellow | Red |
| --- | --- | --- | --- |
| 首次通过率 | `>= 80%` | `50% ~ 79%` | `< 50%` |
| 阶段内逃逸率 | `<= 20%` | `21% ~ 40%` | `> 40%` |
| 平均修复时长 | `<= 1d` | `> 1d 且 <= 3d` | `> 3d` |

## 样本明细
| Sample ID | 来源文档 | 首次结论 | 最终结论 | 是否阶段内逃逸 | 发现日期 | 关闭日期 | 修复时长 | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `LAUNCHREV-20260308` | `doc/testing/launcher/launcher-full-usability-closure-audit-2026-03-08.project.md` | `fail` | `conditional_pass` | `yes` | `2026-03-08` | `2026-03-08` | `0d` | Web 静态目录协议错配风险在可用性审查阶段暴露，并在同日切换显式静态目录后收口。 |
| `TEST-BUNDLE-GAME-018-20260310` | `doc/testing/evidence/release-evidence-bundle-task-game-018-2026-03-10.md` | `pass` | `pass` | `no` | `2026-03-10` | `2026-03-10` | `0d` | task 级 S6 / 回归 / playability 证据一次通过。 |
| `RT-STORAGE-GATE-SAMPLE-20260310-234544` | `doc/world-runtime/evidence/runtime-storage-gate-sample-2026-03-10.md` | `fail` | `pass_after_fix` | `yes` | `2026-03-10` | `2026-03-11` | `1d` | QA 复验发现 storage profile cadence 未真正透传到 execution bridge，次日修复并通过。 |

## 计算结果
- 首次通过率：`1 / 3 = 33.3%` → `red`
- 最终收口率：`3 / 3 = 100%` → `green`
- 阶段内逃逸率：`2 / 3 = 66.7%` → `red`
- 平均修复时长：`(0d + 0d + 1d) / 3 = 0.33d` → `green`

## 基线解读
- 当前 testing 体系的“最终能收口”能力已经可用，但“首次就过”的能力明显不足，说明 QA / 可用性复验仍在承担较高的下游拦截成本。
- 阶段内逃逸率偏高，主要由启动器可用性口径漂移和 runtime profile 透传缺口构成；这类问题都不是线上事故，但说明实现 owner 的自检边界仍需前移。
- 修复时长保持在 `1d` 以内，表明一旦问题暴露，修复协作链路是通的。

## 建议动作
- `producer_system_designer`：阶段评审优先关注“首次通过率”与“阶段内逃逸率”，不要只看最终 `pass`。
- `qa_engineer`：后续 baseline 继续保留 `首次结论/最终结论` 双字段，避免把返工成本隐藏到最终收口率里。
- 模块 owner：对涉及 launcher/runtime 的高风险任务，在提交前补最小 task 级 evidence，自行吸收一轮 QA 才暴露的问题。
