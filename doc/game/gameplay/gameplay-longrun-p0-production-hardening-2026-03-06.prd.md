# Gameplay Long-Run P0 Production Hardening（2026-03-06）

审计轮次: 1

- 对应项目管理文档: `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.project.md`

## 1. Executive Summary
- Problem Statement: 长期在线的区块链 + P2P 多人模拟在高并发与对抗环境下，易出现状态分叉、作弊放大、经济失衡与运维失控，现有文档缺少统一 P0 基线。
- Proposed Solution: 新增 `PRD-GAME-006`，把状态权威分层、确定性回放/回滚、反作弊与反女巫、经济闭环、可运维性收敛为单一验收面。
- Success Criteria:
  - SC-006-1: P2P 传播层与权威裁决层职责边界清晰，冲突写入误放过率为 0。
  - SC-006-2: 同输入回放一致率 100%，回放漂移触发回滚流程成功率 100%。
  - SC-006-3: P0 作弊事件具备“检测 -> 惩罚 -> 申诉 -> 复核”全链路证据，漏检率持续下降。
  - SC-006-4: 经济源汇（日增发/日销毁/净流量）可审计，异常阈值触发告警时延 <= 5 分钟。
  - SC-006-5: 关键运维 SLO、灰度与灾备演练形成发布阻断门禁，演练周期满足周/月要求。

## 2. User Experience & Functionality
- User Personas:
  - 运行值守/SRE：关注稳定性、恢复时长、故障止损。
  - 治理与安全评审者：关注作弊风险、权限滥用与审计证据完整性。
  - 经济设计者：关注产出/消耗平衡、通胀与套利异常。
  - 核心玩法开发者：关注规则变更与运行时行为的一致性。
- User Scenarios & Frequency:
  - 权威冲突审计：按日巡检 + 发布前强制检查。
  - 回放/回滚演练：每周 smoke、每月 full-chaos。
  - 对抗回归（反作弊/反女巫）：每个候选版本至少 1 次。
  - 经济源汇对账：每天 1 次基线审计，异常时即时升级。
  - 运维门禁评审：每次发布执行 D/RC/D-1/D0 节奏。
- User Stories:
  - PRD-GAME-006-01: As a runtime operator, I want authority-layered state commitment, so that non-authoritative writes cannot fork the world.
  - PRD-GAME-006-02: As a SRE, I want deterministic replay and rollback runbooks, so that drift incidents can be recovered quickly.
  - PRD-GAME-006-03: As a security reviewer, I want anti-cheat and anti-sybil evidence pipelines, so that adversarial behavior is accountable.
  - PRD-GAME-006-04: As an economy designer, I want source-sink accounting and anomaly gates, so that inflation and exploit loops stay bounded.
  - PRD-GAME-006-05: As an on-call owner, I want observability and disaster drills with release blockers, so that long-run uptime is sustainable.
- Critical User Flows:
  1. Flow-006-01: `action 传播 -> 权威节点裁决 -> cert 写入 -> 非权威冲突拒绝 -> 审计事件归档`
  2. Flow-006-02: `检测回放漂移 -> 锁定影响区间 -> 执行快照回滚 -> 重放追平 -> 验证 state_root`
  3. Flow-006-03: `作弊检测命中 -> 风险分级 -> 限权/惩罚 -> 申诉 -> 治理复核`
  4. Flow-006-04: `经济日审计 -> 阈值超限 -> 自动降载/冻结高风险入口 -> 发起治理修复提案`
  5. Flow-006-05: `告警触发 -> 值守定位 -> runbook 处置 -> 灰度/回滚 -> 复盘输出`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 状态权威分层 | `authority_source`、`consensus_height`、`block_hash`、`state_root` | 接收传播、执行裁决、拒绝非权威提交 | `propagated -> certified -> committed/rejected` | 先高度后哈希，冲突按权威优先 | 仅权威裁决角色可提交最终状态 |
| 回放与回滚 | `snapshot_id`、`replay_from_tick`、`mismatch_tick`、`rollback_ticket` | 执行回放、创建回滚单、确认恢复 | `detected -> replaying -> rollbacking -> recovered` | 以最小影响区间优先恢复 | 回滚需双人审批（值守+治理） |
| 反作弊与反女巫 | `risk_score`、`evidence_hash`、`penalty_level`、`appeal_id` | 标记风险、执行惩罚、提交申诉、复核裁决 | `suspected -> penalized -> appealed -> resolved` | 风险分由行为/资金/拓扑综合计算 | 安全角色可惩罚，治理角色可复核 |
| 经济闭环审计 | `mint_total`、`burn_total`、`net_flow`、`exploit_signature` | 每日对账、阈值告警、触发防护策略 | `normal -> warning -> protected -> recovered` | 以净流量与环比偏差计算风险 | 经济策略改动需治理通过 |
| 可运维性门禁 | `slo`、`error_budget`、`alert_id`、`drill_id`、`rollback_id` | 告警确认、灰度推进、灾备演练、发布阻断 | `healthy -> degraded -> mitigated -> verified` | P0 告警优先级最高 | 发布负责人可放行，需证据齐全 |
- Acceptance Criteria:
  - AC-006-01: 任意非权威来源提交最终状态时，系统必须拒绝并产出审计事件。
  - AC-006-02: 回放漂移事件必须可定位到 `mismatch_tick`，并在 10 分钟内完成标准回滚流程。
  - AC-006-03: 反作弊/反女巫链路必须保留可验证证据哈希，惩罚与申诉状态机可重放。
  - AC-006-04: 经济审计报表必须输出 `mint/burn/net_flow` 三元指标并保留历史趋势。
  - AC-006-05: 发布前必须完成 SLO 对账、关键告警清零或豁免审批、灾备演练记录。
  - AC-006-06: `PRD-GAME-006-*` 全部映射到任务与 `test_tier_required/full` 命令，追踪可落地执行。
- Non-Goals:
  - 不在本 PRD 中定义具体链上代币参数与手续费数值。
  - 不覆盖前端治理页面的视觉与交互细节。
  - 不替代网络层传输协议底层实现文档。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为运行时/治理/运维策略基线，不引入新增 AI 推理组件）。
- Evaluation Strategy: 以回放一致率、作弊漏检趋势、经济异常告警命中率、SLO 达标率评估成效。

## 4. Technical Specifications
- Architecture Overview:
  - `传播层` 负责动作/事件扩散，不具备最终提交权。
  - `裁决层` 负责共识证书与最终状态提交。
  - `恢复层` 负责回放校验、快照回滚与状态追平。
  - `风控层` 负责作弊/女巫识别与惩罚申诉状态机。
  - `运维层` 负责监控、告警、灰度、灾备与发布阻断。
- Integration Points:
  - `doc/game/prd.md`
  - `doc/game/prd.project.md`
  - `doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
  - `doc/world-runtime/prd.md`
  - `testing-manual.md`
  - `scripts/p2p-longrun-soak.sh`
- Edge Cases & Error Handling:
  - 网络分区：在权威来源不可达期间禁止非权威提交，转入只读保护模式。
  - 空事件批次：允许空 tick 提交，但必须保持证书链连续。
  - 裁决超时：超时后进入降级模式并触发值守升级告警。
  - 并发仲裁冲突：同高度多裁决候选时仅保留权威证书通过路径。
  - 数据损坏：快照校验失败时禁止加载并自动回退上一稳定点。
  - 申诉超窗：超过申诉窗口的惩罚请求仅允许治理层人工复核入口。
  - 经济噪声误报：低置信异常进入观察态，不触发全局冻结。
- Non-Functional Requirements:
  - NFR-006-1: 权威冲突误放过率 = 0。
  - NFR-006-2: 回放一致率 = 100%（同输入同版本）。
  - NFR-006-3: 回滚演练周成功率 >= 95%，月 full-chaos 成功率 >= 90%。
  - NFR-006-4: 作弊风险事件证据完整率 = 100%。
  - NFR-006-5: 经济日审计任务成功率 = 100%，关键异常告警延迟 <= 5 分钟。
  - NFR-006-6: P0 告警 MTTA <= 5 分钟，MTTR <= 30 分钟（按周统计）。
- Security & Privacy:
  - 惩罚与申诉证据使用哈希/摘要存档，避免暴露敏感原始数据。
  - 紧急权限动作需签名、双人审批与审计留痕，不允许单点绕过。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 固化 P0 文档基线与任务映射（T0）。
  - v1.1: 落地权威分层 + 回放回滚演练门禁（T1/T2）。
  - v2.0: 落地风控、经济审计、运维门禁并完成长稳验证（T3/T4/T5）。
- Technical Risks:
  - 风险-006-1: 权威门禁过严可能导致短时可用性下降。
  - 风险-006-2: 回滚策略不当可能放大数据回退范围。
  - 风险-006-3: 反女巫策略阈值过高会误伤正常活跃玩家。
  - 风险-006-4: 经济告警阈值配置错误可能引发频繁误报。

## 5.1 当前实现切片（2026-03-06）
- 已完成 `TASK-GAME-013`（`PRD-GAME-006-01`）：
  - 在 `TickCertificate` 增加 `authority_source` 与 `submission_role`（`propagation/authority`）字段，用于区分传播提交与权威裁决提交。
  - `World` 增加权威源配置 `tick_consensus_authority_source`，默认绑定 `builtin.module.release.signer`，并提供显式切换接口。
  - 新增提交接口：
    - `record_tick_consensus_propagation_for_tick`
    - `record_tick_consensus_authority_for_tick`
  - 落地冲突仲裁拒绝规则：
    - 已有权威提交时，非权威重复提交一律拒绝。
    - 未配置为当前权威源的 `authority` 提交一律拒绝。
    - 不同传播源在同 tick 形成冲突哈希时，必须等待权威裁决。
  - 新增审计事件模型 `TickConsensusRejectionAuditEvent`，记录 `attempted_source/attempted_role/existing_source/existing_role/reason`。
  - 快照与持久化链路已接线：`snapshot/save/load/from_snapshot` 保持权威配置与拒绝审计事件可恢复。
- 已通过定向回归：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::basic::tick_consensus_ -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::persistence::persist_and_restore_world -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::basic::from_snapshot_replay_rebuilds_missing_tick_consensus_records -- --nocapture`

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-GAME-006-01 | TASK-GAME-013 | `test_tier_required` | 权威冲突注入测试 + 非权威写入拒绝断言 | 共识提交与状态一致性 |
| PRD-GAME-006-02 | TASK-GAME-014 | `test_tier_required` + `test_tier_full` | 回放漂移注入、快照回滚演练、恢复后 `state_root` 对账 | 恢复能力与数据完整性 |
| PRD-GAME-006-03 | TASK-GAME-015 | `test_tier_required` + `test_tier_full` | 对抗样本回放、惩罚申诉状态机回归、证据链完整性校验 | 安全与治理公平性 |
| PRD-GAME-006-04 | TASK-GAME-016 | `test_tier_required` | 经济源汇日审计、阈值越线告警、自动保护策略触发验证 | 经济稳定性与抗套利能力 |
| PRD-GAME-006-05 | TASK-GAME-017 | `test_tier_required` | SLO 对账、告警演练、灰度与灾备回滚演练 | 可运维性与发布稳定性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-006-01 | 先定义权威分层再扩展性能优化 | 先追求低延迟再补一致性约束 | 长期在线系统以正确性优先，防止分叉累积。 |
| DEC-006-02 | 回放漂移强制回滚 runbook 化 | 人工临时处理单次故障 | runbook 能降低值守误操作并提升恢复确定性。 |
| DEC-006-03 | 反作弊与反女巫统一证据链 | 风控与治理分散记录 | 统一证据链便于审计、申诉与追责闭环。 |
| DEC-006-04 | 经济源汇纳入发布门禁 | 仅做离线观察不阻断发布 | 经济异常会直接破坏长期服可信性，需强门禁。 |
