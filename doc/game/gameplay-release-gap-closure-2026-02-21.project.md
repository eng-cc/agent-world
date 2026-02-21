# Gameplay 发行差距收口（项目管理文档）

## 任务拆解

### T0 文档建模
- [x] 新建设计文档：`doc/game/gameplay-release-gap-closure-2026-02-21.md`
- [x] 新建项目管理文档：`doc/game/gameplay-release-gap-closure-2026-02-21.project.md`

### T1 Prompt 触发优化（对应问题 1）
- [x] 优化默认 LLM `system_prompt` / `short_term_goal` / `long_term_goal`（改为游戏发展导向，去除强制动作链）
- [x] 增加中途切换 prompt 能力（demo + stress 透传）
- [x] 增加游戏发展测试 prompt 套件（`--prompt-pack`：`story_balanced/frontier_builder/civic_operator/resilience_drill`）
- [x] 增加定向测试，验证切换前后动作覆盖变化

### T2 Gate 覆盖补齐与复验（对应问题 2/3）
- [x] 扩展 `llm-longrun-stress.sh`：release gate profile（industrial/gameplay/hybrid）
- [x] 更新 `testing-manual.md` S8 发行口径示例
- [x] 跑 long-run 复验并记录 summary/report 产物

### T3 经济治理动作协议补齐（对应问题 4）
- [x] 扩展 OpenAI decision schema 增加经济合约动作
- [x] 扩展 parser 到 `Action::Open/Accept/SettleEconomicContract`
- [x] 补齐 `test_tier_required` 解析/边界测试

### T4 m5 规则深度优化（对应问题 5）
- [x] 增强 `m5_gameplay_war_core` 结算规则输出
- [x] 增强 `m5_gameplay_crisis_cycle` 动态危机生成/超时规则
- [x] 增强 `m5_gameplay_economic_overlay` 的经济事件联动
- [x] 补齐 gameplay 协议测试断言（required/full）

### T5 回归与收口
- [x] 运行定向测试与脚本回归（含 gate）
- [x] 回写项目文档状态
- [x] 回写 `doc/devlog/2026-02-21.md`

### T6 千 tick 长周期演进（新增）
- [x] `world_llm_agent_demo` 增加多阶段切换参数（`--prompt-switches-json`）并与单次切换参数做互斥校验
- [x] `llm-longrun-stress.sh` 支持透传 `--prompt-switches-json`
- [x] `story_balanced` 在长周期（中长/超长）自动生成多阶段切换计划
- [x] 更新 `testing-manual.md` 长周期示例与参数说明
- [x] 运行解析/脚本回归并记录日志

### T7 llm_bootstrap 多 Agent + runtime gameplay bridge（新增）
- [x] `llm_bootstrap` 场景从 1 Agent 扩展到 5 Agent（新增 4 个 Agent）
- [x] `world_llm_agent_demo` 增加 runtime gameplay bridge（接管 runtime-only gameplay/economic 动作）
- [x] `llm-longrun-stress.sh` 增加 bridge 参数透传与默认策略
- [x] 补齐 demo/bin 测试与脚本回归，验证 gameplay 动作拒绝率下降
- [x] 更新 `testing-manual.md` 闭环测试建议（5 Agent + bridge）

## 依赖
- `doc/game/gameplay-release-production-closure.md`
- `doc/game/gameplay-module-driven-production-closure.md`
- `testing-manual.md`
- `scripts/llm-longrun-stress.sh`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1、T2、T3、T4、T5、T6、T7
- 本轮增量：`llm_bootstrap` 升级为 5 Agent，并在 demo/stress 路径接入 runtime gameplay bridge，消除 runtime-only gameplay/economic 动作在 simulator 路径上的非预期拒绝。
- 进行中：基于 5 Agent + bridge 的玩法覆盖稳定性调优（以预设世界事件/阶段目标作为下一轮候选）。
- 阻塞项：无
