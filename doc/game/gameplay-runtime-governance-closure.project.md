# Gameplay Runtime Governance Closure（项目管理文档）

## 任务拆解

### T0 设计建档
- [x] 输出设计文档：`doc/game/gameplay-runtime-governance-closure.md`
- [x] 输出项目管理文档：`doc/game/gameplay-runtime-governance-closure.project.md`

### T1 ABI 与角色模型
- [x] 新增 `GameplayModuleKind` 与 `GameplayContract`
- [x] 扩展 `ModuleRole::Gameplay`
- [x] 扩展 `ModuleAbiContract` 挂载 gameplay 元数据

### T2 Runtime 治理校验
- [x] 校验 `role` 与 `abi_contract.gameplay` 一致性
- [x] 校验 gameplay 元数据字段合法性（mode/min/max）
- [x] 在模块变更闭环中新增 `(game_mode, gameplay_kind)` 激活冲突检测

### T3 Runtime 可观测与验收
- [x] 新增 `active gameplay modules` 查询接口
- [x] 新增 `gameplay mode readiness` 报告接口（coverage/missing）
- [x] 新增 runtime 单测覆盖（合法/拒绝/冲突/就绪度）

### T4 回归与文档收口
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::gameplay:: -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p agent_world --lib`
- [x] 回写任务状态、devlog 与结论

## 依赖
- `doc/game/gameplay-engineering-architecture.md`
- `doc/game/gameplay-top-level-design.md`
- `testing-manual.md`

## 状态
- 当前状态：`已完成（首个生产切片已收口）`
- 阻塞项：无
