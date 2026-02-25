# Non-Viewer 设计一致性修复（项目管理）

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/nonviewer/nonviewer-design-alignment-fixes-2026-02-25.md`
- [x] 新建项目管理文档：`doc/nonviewer/nonviewer-design-alignment-fixes-2026-02-25.project.md`

### T1 修复 rejected 分支静默丢失
- [x] `PosNodeEngine::apply_decision` rejected 分支回灌失败改为显式报错
- [x] `pending_consensus_action_capacity` 增加 pending proposal 回灌容量预留
- [x] `agent_world_node` 定向回归测试通过

### T2 修复 dead-letter 冷归档回放缺口
- [x] `FileMembershipRevocationAlertDeadLetterStore::list` 改为冷热聚合读取
- [x] `FileMembershipRevocationAlertDeadLetterStore::list_delivery_metrics` 改为冷热聚合读取
- [x] `replace` 清理并重建 archive refs，避免 stale cold refs
- [x] `agent_world_consensus` 定向回归测试通过

### T3 收口
- [x] 回写 `doc/nonviewer/README.md` 活跃文档索引
- [x] 追加 `doc/devlog/2026-02-25.md` 任务日志

## 依赖
- `crates/agent_world_node/src/lib_impl_part1.rs`
- `crates/agent_world_node/src/tests_action_payload.rs`
- `crates/agent_world_consensus/src/membership_recovery/dead_letter.rs`
- `crates/agent_world_consensus/src/membership_recovery_tests_split_part1.rs`

## 状态
- 当前状态：已完成
- 已完成：T0、T1、T2、T3
- 进行中：无
- 未开始：无
- 阻塞项：无
