# 游戏可玩性顶层设计（项目管理文档）

- 对应设计文档: `doc/game/gameplay/gameplay-top-level-design.design.md`
- 对应需求文档: `doc/game/gameplay/gameplay-top-level-design.prd.md`
审计轮次: 4

## ROUND-002 主从口径
- 本文件为 gameplay 项目主入口，其余 gameplay project 为增量计划。

## 任务拆解

### T0 文档与结构对齐
- [x] 将顶层设计文档迁移到 `doc/game/`：`doc/game/gameplay/gameplay-top-level-design.prd.md`
- [x] 将工程设计分册迁移并重命名为语义化文件：`doc/game/gameplay/gameplay-engineering-architecture.md`
- [x] 修复工程设计分册 Markdown 围栏问题，确保文档可正常渲染

### T1 顶层设计字段补齐
- [x] 在顶层设计文档中补齐必备字段：目标、范围、接口/数据、里程碑、风险
- [x] 在工程设计分册中补齐范围、接口/数据、里程碑、风险

### T2 设计评审准备
- [x] 组织一次可玩性评审，确认微/中/长循环是否可验证
- [x] 将“爽点曲线”映射为可量化指标（留存、冲突频次、联盟活跃度）
- [x] 对战争与政治机制补充最小可行数值基线（成本/收益/冷却约束）

### T3 工程落地拆解（下阶段）
- [x] 落地 Gameplay Runtime 治理闭环首个生产切片（`doc/game/gameplay/gameplay-runtime-governance-closure.prd.md`）：ABI gameplay 元数据、Runtime 校验、mode+kind 槽位冲突检测、就绪度报告与测试
- [x] 拆解 WASM Gameplay Kernel API 的实现任务（读取/提案/事件总线），并落地生命周期规则切片（`doc/game/gameplay/gameplay-layer-lifecycle-rules-closure.prd.md`）
- [x] 拆解 War/Governance/Crisis/Economic/Meta 模块 MVP 任务，并完成协议与模块生产实现（`doc/game/gameplay/gameplay-layer-war-governance-crisis-meta-closure.prd.md`、`doc/game/gameplay/gameplay-module-driven-production-closure.prd.md`）
- [x] 为每个模块定义 `test_tier_required` 与 `test_tier_full` 测试矩阵（见下文“Gameplay 模块测试矩阵引用”）

### T4 前期工业引导闭环（2026-03-15）
- [x] 冻结“首个制成品 -> 首条稳定生产链 -> 首座工厂单元 -> 可交易工业品 -> 受保护工业节点”作为新手前期主引导链。
- [x] 将前 30 天体验路径改写为“工业成长优先，联盟/治理/战争后接”，并同步评审与指标口径。
- [x] `runtime_engineer`：补齐工业里程碑所需的生产完成、停机、恢复状态与审计事件，确保结果可由状态与事件历史解释。
- [x] `viewer_engineer`：把 `已接受 / 执行中 / 已产出 / 停机原因` 做成主界面显式反馈，优先覆盖首个制成品与工厂开工场景。
- [x] `qa_engineer`：新增“首个制成品 / 停机恢复 / 首座工厂单元”playability 卡片与 `test_tier_required` 手动回归链路。

### T5 PostOnboarding 阶段目标链（2026-03-18）
- [x] 冻结 `FirstSessionLoop -> PostOnboarding -> MidLoop` 的阶段承接口径，并新增专题 PRD / design / project。
- [ ] `viewer_engineer` / `runtime_engineer`：对齐 `PostOnboarding` 阶段机、主目标来源、阻塞分类与恢复逻辑。
- [ ] `viewer_engineer`：落地阶段切换卡、主目标卡、阶段完成卡，关闭当前 `#46` 的产品承接缺口。
- [ ] `qa_engineer`：新增 `#46` required-tier / Web 闭环与 playability 卡片证据，形成通过或阻断结论。

### T6 纯 API 客户端等价（2026-03-19）
- [x] 冻结“纯 API 客户端在信息粒度、动作能力和持续游玩上与 UI 等价”专题 PRD / design / project。
- [ ] `viewer_engineer` / `runtime_engineer`：将关键玩家语义从 UI 私有组装下沉到协议级 canonical snapshot。
- [ ] `runtime_engineer` / `agent_engineer` / `viewer_engineer`：补齐纯 API 正式玩家动作面与恢复逻辑，避免降级为 observer-only。
- [ ] `qa_engineer`：建立 UI/API parity matrix 与纯 API 长玩 required/full 验收。

## 依赖

- 运行时与模块治理基线：`doc/world-runtime/prd.md`
- 测试流程与分层矩阵：`testing-manual.md`
- 世界规则与边界约束：`world-rule.md`
- 战争与政治数值基线：`doc/game/gameplay/gameplay-war-politics-mvp-baseline.md`

## Gameplay 模块测试矩阵引用

- `test_tier_required` 基线：`./scripts/ci-tests.sh required`（来源：`testing-manual.md` S1）
- `test_tier_full` 基线：`./scripts/ci-tests.sh full`（来源：`testing-manual.md` S2）
- Gameplay Runtime 协议定向：`env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::gameplay_protocol:: -- --nocapture`（来源：`testing-manual.md` S3）
- Gameplay LLM/Simulator 协议定向：
  - `env -u RUSTC_WRAPPER cargo test -p oasis7 simulator::llm_agent::tests:: -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p oasis7 simulator::tests::submitter_access:: -- --nocapture`
- 场景回归入口：`env -u RUSTC_WRAPPER cargo test -p oasis7 --features test_tier_required scenario_specs_match_ids -- --nocapture`（来源：`testing-manual.md` S7）

## 状态

- 当前状态：`进行中`
- 已完成：文档归位、命名语义化、必备字段补齐、工程分册格式修复、Gameplay Runtime/模块化/协议扩展任务拆解与落地、Gameplay 模块测试矩阵引用固化、设计评审准备与战争/政治数值基线补齐、前期工业引导闭环文档冻结（首个制成品/工厂主链）、T4 的 runtime 工业状态/事件与 viewer 主反馈闭环、T5 的 `PostOnboarding` 阶段目标链文档冻结与根入口挂载、T6 的纯 API 客户端等价专题冻结与根入口挂载。
- 未完成：`T6` 的协议级 canonical 玩家语义、正式玩家动作面与 parity 验收尚未实现。
- 阻塞项：无（待相关 owner 按 `T6` 执行）

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
