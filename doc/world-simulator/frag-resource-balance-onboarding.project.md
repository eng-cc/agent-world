# Agent World Simulator：Frag 资源平衡与新手友好生成（项目管理文档）

## 任务拆解
- [x] FRB1 输出设计文档与项目管理文档
- [x] FRB2.1 扩展 `AsteroidFragmentConfig`（保底数量 + 中心引导参数）
- [x] FRB2.2 在 `generate_fragments` 接入中心区密度倍率与确定性补种
- [x] FRB2.3 补充 `test_tier_required` 测试（数量保底/分布偏置/sanitize）
- [ ] FRB3.1 回写 `doc/world-simulator.project.md` 状态
- [ ] FRB3.2 记录当日 devlog（日期/完成内容/遗留事项）
- [ ] FRB3.3 运行回归测试并完成收口

## 依赖
- `crates/agent_world/src/simulator/world_model.rs`
- `crates/agent_world/src/simulator/asteroid_fragment.rs`
- `crates/agent_world/src/simulator/tests/asteroid_fragment.rs`
- `doc/world-simulator.project.md`
- `doc/devlog/`

## 状态
- 当前阶段：FRB3（回归与文档收口）
- 下一阶段：FRB 收口提交
