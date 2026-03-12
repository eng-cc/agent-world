# OpenClaw vs 内置 Agent parity 聚合结论模板（2026-03-12）

审计轮次: 1

适用范围: `PRD-WORLD_SIMULATOR-038` 的 `T2/T4/T5`，用于汇总自动 benchmark、trace 统计和 QA/producer 评分卡结论。

---

## 一、批次信息
- benchmark_run_id:
- parity_tier:
- provider_version:
- adapter_version:
- protocol_version:
- 执行日期:
- 执行人:

## 二、样本覆盖
| Scenario-ID | provider | sample_count | valid_samples | invalid_fixture | benchmark_status |
| --- | --- | --- | --- | --- | --- |
| P0-001 | builtin |  |  |  |  |
| P0-001 | openclaw_local_http |  |  |  |  |

## 三、核心指标并排对比
| 指标 | builtin | OpenClaw | gap / 备注 | 是否达标 |
| --- | --- | --- | --- | --- |
| completion_rate |  |  |  | [ ] |
| invalid_action_rate |  |  |  | [ ] |
| timeout_rate |  |  |  | [ ] |
| median_extra_wait_ms |  |  |  | [ ] |
| p95_extra_wait_ms |  |  |  | [ ] |
| trace_completeness |  |  |  | [ ] |
| recoverable_error_resolution_rate |  |  |  | [ ] |
| context_drift_count |  |  |  | [ ] |

## 四、失败签名汇总
| error_code | builtin count | OpenClaw count | 是否阻断 | 备注 |
| --- | --- | --- | --- | --- |
| provider_unreachable |  |  | [ ] |  |
| timeout |  |  | [ ] |  |
| invalid_action_schema |  |  | [ ] |  |
| context_drift |  |  | [ ] |  |
| session_cross_talk |  |  | [ ] |  |

## 五、主观评分关联
- QA 评分卡路径:
- Producer 评分卡路径:
- 关键截图/trace 证据路径:
- 自动 benchmark 证据路径:

## 六、阻断项结论
- [ ] 未触发阻断项
- [ ] completion gap 超线
- [ ] timeout / latency 超线
- [ ] trace 不完整导致无法诊断
- [ ] 记忆连续性明显漂移
- [ ] 会话串线 / provider session 污染
- [ ] 其他:

## 七、最终结论
- 结论: [ ]blocked [ ]failed [ ]conditional_pass [ ]pass
- 建议状态: [ ]保持 experimental [ ]进入下一层 parity [ ]允许默认启用
- 必修项:
- 建议优化项:
- 备注:
