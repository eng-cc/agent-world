# Gameplay Layer War/Governance/Crisis/Meta Closure（项目管理文档）

## 任务拆解

### T0 设计建档
- [x] 新建设计文档：`doc/game/gameplay-layer-war-governance-crisis-meta-closure.md`
- [x] 新建项目管理文档：`doc/game/gameplay-layer-war-governance-crisis-meta-closure.project.md`

### T1 协议与状态实现
- [x] 扩展 `events.rs` 动作/领域事件原语（联盟/战争/投票/危机/元进度）
- [x] 扩展 `event_processing.rs` 动作校验与事件映射
- [x] 扩展 `state.rs` 持久化状态与事件应用
- [x] 扩展 `module_runtime_labels.rs` 标签映射与相关导出

### T2 WASM Gameplay 模块与启动
- [x] 新增 5 个 Gameplay builtin wasm 模块（war/governance/crisis/economic/meta）
- [x] 新增 Gameplay builtin artifact loader 与清单
- [x] 新增 `bootstrap_gameplay.rs` 并接入 world 模块
- [x] 场景启动链路覆盖 gameplay + economy + 基础层

### T3 测试与收口
- [x] 新增协议与状态单测（`test_tier_required`）
- [x] 新增 gameplay bootstrap/wasm 测试（`test_tier_full`）
- [x] 运行关键测试命令并通过
- [x] 回写项目文档状态与 `doc/devlog/2026-02-20.md`

## 依赖
- `README.md`
- `doc/game/gameplay-top-level-design.md`
- `doc/game/gameplay-engineering-architecture.md`
- `doc/game/gameplay-base-runtime-wasm-layer-split.md`
- `testing-manual.md`

## 状态
- 当前状态：`已完成`
- 已完成：T0、T1、T2、T3
- 进行中：无
- 阻塞项：无
