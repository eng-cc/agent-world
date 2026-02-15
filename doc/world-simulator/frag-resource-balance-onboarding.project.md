# Agent World Simulator：Frag 资源平衡与新手友好生成（项目管理文档）

## 任务拆解
- [x] FRB1 输出设计文档与项目管理文档
- [x] FRB2.1 扩展 `AsteroidFragmentConfig`（保底数量 + 中心引导参数）
- [x] FRB2.2 在 `generate_fragments` 接入中心区密度倍率与确定性补种
- [x] FRB2.3 补充 `test_tier_required` 测试（数量保底/分布偏置/sanitize）
- [x] FRB3.1 回写 `doc/world-simulator.project.md` 状态
- [x] FRB3.2 记录当日 devlog（日期/完成内容/遗留事项）
- [x] FRB3.3 运行回归测试并完成收口
- [x] FRB4.1 更新设计文档：补种改为“每 100 tick 补 1%”与资源类型分布策略
- [x] FRB4.2 扩展 `AsteroidFragmentConfig`（`replenish_interval_ticks` / `replenish_percent_ppm` / `material_distribution_strategy`）
- [x] FRB4.3 在 `WorldKernel::step` 接入运行期补种与 `FragmentsReplenished` 事件（含 replay）
- [x] FRB4.4 在 `generate_fragments` 接入资源类型分区分布策略
- [x] FRB4.5 补充 `test_tier_required` 测试（补种周期、补种比例、分布策略、replay）
- [ ] FRB4.6 回写总项目文档与 devlog，完成收口

## 依赖
- `crates/agent_world/src/simulator/world_model.rs`
- `crates/agent_world/src/simulator/asteroid_fragment.rs`
- `crates/agent_world/src/simulator/kernel/*`
- `crates/agent_world/src/simulator/tests/asteroid_fragment.rs`
- `crates/agent_world/src/simulator/tests/kernel.rs`
- `crates/agent_world/src/simulator/tests/persist.rs`
- `doc/world-simulator.project.md`
- `doc/devlog/`

## 状态
- 当前阶段：FRB4（收口中）
- 下一阶段：FRB4 收口提交
