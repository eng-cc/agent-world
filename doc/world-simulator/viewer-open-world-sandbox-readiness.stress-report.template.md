# Viewer OWR4 压测报告模板（triad/llm）

## 基本信息
- 日期：
- 执行人：
- 分支 / commit：
- 机器配置（CPU/GPU/内存/OS）：

## 压测命令
```bash
./scripts/viewer-owr4-stress.sh \
  --duration-secs 45 \
  --tick-ms 120 \
  --scenarios triad_region_bootstrap,llm_bootstrap
```

## 场景矩阵
| 场景 | 决策模式 | 目标 |
|---|---|---|
| `triad_region_bootstrap` | script | 区域规模对象压力（基础高负载） |
| `llm_bootstrap` | llm | 事件与决策链路压力（LLM 高负载） |

## 结果汇总（脚本输出）
| 场景 | mode | duration(s) | tick(ms) | final_events | events/s | viewer status |
|---|---:|---:|---:|---:|---:|---|
| triad_region_bootstrap |  |  |  |  |  |  |
| llm_bootstrap |  |  |  |  |  |  |

## 渲染指标（人工补录）
> 从 viewer 右侧性能摘要或回放记录补录。

| 场景 | frame_ms_avg | frame_ms_p95 | visible_labels | overlay_entities | event_window_size | auto_degrade_level_peak |
|---|---:|---:|---:|---:|---:|---:|
| triad_region_bootstrap |  |  |  |  |  |  |
| llm_bootstrap |  |  |  |  |  |  |

## 异常与退化记录
- triad：
- llm：

## 结论
- 是否满足 OWR4 阶段目标：
- 风险项：
- 下一步动作：
