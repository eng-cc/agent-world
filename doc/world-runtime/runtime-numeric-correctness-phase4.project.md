# Agent World Runtime：Replication Writer Epoch/Sequence 数值语义硬化（15 点清单第四阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase4.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase4.project.md`

### T1 Replication Writer 递进显式溢出语义
- [ ] `next_local_record_position` 改为可失败接口，移除关键路径 `saturating_add`。
- [ ] `build_local_commit_message` 透传位置计算失败错误。
- [ ] 新增 replication 模块溢出边界测试（3 类路径）。

### T2 回归与收口
- [ ] 运行 node 定向测试并确认 required-tier 门禁通过。
- [ ] 更新设计/项目文档状态与 `doc/devlog/2026-02-23.md` 收口记录。

## 依赖
- `crates/agent_world_node/src/replication.rs`
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/tests.rs`
- `crates/agent_world_node/src/tests_hardening.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2
- 阻塞项：无
