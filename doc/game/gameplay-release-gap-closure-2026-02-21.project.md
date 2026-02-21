# Gameplay 发行差距收口（项目管理文档）

## 任务拆解

### T0 文档建模
- [x] 新建设计文档：`doc/game/gameplay-release-gap-closure-2026-02-21.md`
- [x] 新建项目管理文档：`doc/game/gameplay-release-gap-closure-2026-02-21.project.md`

### T1 Prompt 触发优化（对应问题 1）
- [ ] 优化默认 LLM `system_prompt` / `short_term_goal` / `long_term_goal`
- [ ] 增加中途切换 prompt 能力（demo + stress 透传）
- [ ] 增加定向测试，验证切换前后动作覆盖变化

### T2 Gate 覆盖补齐与复验（对应问题 2/3）
- [ ] 扩展 `llm-longrun-stress.sh`：release gate profile（industrial/gameplay/hybrid）
- [ ] 更新 `testing-manual.md` S8 发行口径示例
- [ ] 跑 long-run 复验并记录 summary/report 产物

### T3 经济治理动作协议补齐（对应问题 4）
- [ ] 扩展 OpenAI decision schema 增加经济合约动作
- [ ] 扩展 parser 到 `Action::Open/Accept/SettleEconomicContract`
- [ ] 补齐 `test_tier_required` 解析/边界测试

### T4 m5 规则深度优化（对应问题 5）
- [ ] 增强 `m5_gameplay_war_core` 结算规则输出
- [ ] 增强 `m5_gameplay_crisis_cycle` 动态危机生成/超时规则
- [ ] 增强 `m5_gameplay_economic_overlay` 的经济事件联动
- [ ] 补齐 gameplay 协议测试断言（required/full）

### T5 回归与收口
- [ ] 运行定向测试与脚本回归（含 gate）
- [ ] 回写项目文档状态
- [ ] 回写 `doc/devlog/2026-02-21.md`

## 依赖
- `doc/game/gameplay-release-production-closure.md`
- `doc/game/gameplay-module-driven-production-closure.md`
- `testing-manual.md`
- `scripts/llm-longrun-stress.sh`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 阻塞项：无
