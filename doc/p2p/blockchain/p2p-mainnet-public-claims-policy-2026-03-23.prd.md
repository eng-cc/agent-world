# oasis7 mainnet/public claims policy 复评（2026-03-23）

- 对应设计文档: `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.design.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.project.md`

审计轮次: 1
## 1. Executive Summary
- Problem Statement: `MAINNET-1~3` 已经完成规格 gate 建档，但它们当前仍只是 producer 级 source-of-truth 和放行标准，不是 runtime 已完成的生产实现。如果不在此时做一次正式 public claims re-evaluation，团队仍可能把“规格定义好了”误说成“已经对标主流主链”或“创世已 production ready”。
- Proposed Solution: 建立 `MAINNET-4` public claims policy 专题，基于 `MAINNET-1~3` 的当前真值与未完成工程项，冻结允许口径、禁止口径、升级条件和复评规则，并把当前总 verdict 再次锁定为 `not_mainnet_grade`。
- Success Criteria:
  - SC-1: 明确当前允许口径只包括 `limited playable technical preview` 与 `crypto-hardened preview`。
  - SC-2: 明确当前禁止口径至少包括 `mainnet-grade`、`mainstream public-chain-grade security`、`production mint ready`。
  - SC-3: 明确 `MAINNET-1~3` 当前只完成 specification gate，而非工程闭环。
  - SC-4: 给出未来升级 public claims 的最小条件：custody 实装、governance signer 真值外部化实装、genesis freeze/ceremony/QA 实际通过。
  - SC-5: 形成 `CLAIMS-1~4` 任务链，覆盖现状复评、claim allowlist/denylist、升级条件和 liveops 执行规则。

## 2. User Experience & Functionality
- User Personas:
  - `producer_system_designer`：需要一个不会再被随意改写的正式安全/创世口径。
  - `liveops_community`：需要知道哪些词现在能说，哪些词必须绝对禁止。
  - `qa_engineer`：需要把 public claims 和 gate/pass 状态一一绑定。
  - `runtime_engineer`：需要知道哪些实现闭环完成后，项目才有资格重评。
- User Scenarios & Frequency:
  - 每次对外写版本阶段、安全程度或创世准备状态时执行。
  - 每次有新 gate 通过、需要重新评估口径时执行。
  - 每次准备对投资人/社区/合作方做状态同步时执行。
- User Stories:
  - PRD-P2P-CLAIMS-001: As a `producer_system_designer`, I want one final re-evaluation document after `MAINNET-1~3`, so that current claims are grounded in the true completion state.
  - PRD-P2P-CLAIMS-002: As a `liveops_community`, I want an explicit allowlist/denylist of phrases, so that outward communication does not drift upward.
  - PRD-P2P-CLAIMS-003: As a `qa_engineer`, I want claim upgrades tied to concrete gate pass conditions, so that claims are auditable.
  - PRD-P2P-CLAIMS-004: As a `runtime_engineer`, I want the remaining implementation blockers called out clearly, so that the next engineering topics are objective.
- Critical User Flows:
  1. Flow-P2P-CLAIMS-001: `读取 MAINNET-1~3 当前状态 -> 核对哪些只是 spec gate、哪些已工程闭环 -> producer 输出最终复评结论`
  2. Flow-P2P-CLAIMS-002: `liveops 起草对外口径 -> 对照 allowlist/denylist -> 命中禁止词则直接拒绝`
  3. Flow-P2P-CLAIMS-003: `未来某个工程实现完成 -> 对照升级条件 -> 仅在全部条件满足时才允许重新评估更高级口径`
  4. Flow-P2P-CLAIMS-004: `有人提议使用 mainnet-grade/mint-ready 等说法 -> QA/producer 查 claim gate -> 当前若仍有 blocker 则判 block`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算/判定规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Final re-evaluation | `claim_scope/current_verdict/basis/open_blockers` | 基于 MAINNET-1~3 输出最终复评 | `draft -> frozen` | 若任一 blocker 仍存在，则 verdict 维持 `not_mainnet_grade` | `producer_system_designer` 拍板 |
| Claim allowlist | `phrase/min_state/notes` | 定义当前允许对外使用的说法 | `draft -> enforced` | 只能允许与当前真实阶段一致的 phrasing | `liveops_community` 执行 |
| Claim denylist | `phrase/reject_reason` | 定义当前绝对禁止的说法 | `draft -> enforced` | 命中 denylist 直接 block | producer/liveops 联审 |
| Upgrade condition set | `future_claim/required_gates/required_execution_state` | 定义未来想升级口径需要满足的条件 | `draft -> frozen` | spec gate 完成不等于 execution gate 完成 | producer/QA 联审 |
| Engineering blocker handoff | `blocker_id/current_truth/next_owner` | 把剩余工程项交给后续 owner | `listed -> handed_off` | blocker 必须可追溯到具体 topic | producer 维护 |
- Acceptance Criteria:
  - AC-1: 专题必须明确当前总 verdict 仍是 `not_mainnet_grade`。
  - AC-2: 必须明确当前允许口径只有 `limited playable technical preview` 与 `crypto-hardened preview`。
  - AC-3: 必须明确 `mainnet-grade`、`mainstream public-chain-grade security`、`production mint ready` 当前全部禁止。
  - AC-4: 必须明确 `MAINNET-1~3` 当前只完成 specification gate，而不是工程实现闭环。
  - AC-5: 必须列出至少 3 个剩余工程 blocker：`offline storage + manual multisig` custody 实装、governance truth 直接上链实装、genesis freeze/ceremony/QA 实际通过。
  - AC-6: 必须定义未来升级 claim 所需的 execution conditions，而不是只写“后续再评估”。
  - AC-7: 必须输出 `CLAIMS-1~4` 任务链与 owner/test tier 映射。
  - AC-8: 模块主 PRD/project/index/README 与 readiness project 必须接入本专题。
- Non-Goals:
  - 本轮不把任何安全阶段升级到更高档位。
  - 本轮不直接实现剩余工程 blocker。
  - 本轮不重复定义 custody/governance/genesis 细节。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: `MAINNET-4` 不是新技术方案，而是把 `MAINNET-1~3` 的 producer/QA 级结论收束为最终 public claims policy。其核心判断是：当前所有 readiness gate 只完成了规格冻结，尚未完成 production execution，因此 public claims 不得越过 preview 档。
- Integration Points:
  - `doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md`
  - `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.prd.md`
  - `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md`
  - `doc/p2p/blockchain/p2p-genesis-freeze-ceremony-qa-gate-2026-03-23.prd.md`
  - `doc/p2p/blockchain/p2p-mainnet-crypto-security-baseline-2026-03-23.prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 若有人把 spec gate 完成误写成 execution complete，必须直接回滚为 `preview` 口径。
  - 若单个 blocker 工程已完成，但其他 blocker 仍未过，则整体 claim 仍不得升级。
  - 若某轮外部沟通已使用 denylist 词汇，必须记录为口径事故并在下一轮文档中纠偏。
  - 若 future upgrade condition 写得不可验证，视为无效条件，仍不得升级。
- Non-Functional Requirements:
  - NFR-P2P-CLAIMS-1: public claims policy 必须是 hard gate，不允许“酌情放宽”。
  - NFR-P2P-CLAIMS-2: 所有升级条件都必须可验证、可追溯到 topic 和 QA/pass 状态。
  - NFR-P2P-CLAIMS-3: 在 execution blockers 清零前，公开口径上限固定为 `crypto-hardened preview`。
  - NFR-P2P-CLAIMS-4: 任何 denylist 词汇都必须附明确 reject reason，避免二次歧义。
- Security & Privacy: 本专题只定义对外口径和升级条件，不涉及新的敏感数据；所有判断仅引用已有正式文档和 QA 门禁。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成 `MAINNET-4` public claims policy 与最终复评文档。
  - v1.1: 绑定 liveops 对外口径模板与审核流程。
  - v2.0: 待 execution blockers 清零后，按本专题升级条件重新复评。
- Technical Risks:
  - 风险-1: 如果不把 spec gate 和 execution gate 区分开，团队仍会误判当前阶段。
  - 风险-2: 如果 denylist 不冻结，外部沟通会反复出现“接近主流主链”的模糊表述。
  - 风险-3: 如果 future upgrade condition 不可验证，后续仍会回到口头争论。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-CLAIMS-001 | CLAIMS-0/1 | `test_tier_required` | MAINNET-1~3 结论复核、最终 verdict 冻结、模块主追踪回写 | 阶段判断与 producer 口径 |
| PRD-P2P-CLAIMS-002 | CLAIMS-1/2 | `test_tier_required` | claim allowlist/denylist 冻结与 liveops 使用边界核对 | 对外沟通与品牌风险 |
| PRD-P2P-CLAIMS-003 | CLAIMS-2/3 | `test_tier_required` | upgrade condition 与 QA/pass 依赖链核对 | 后续阶段升级门槛 |
| PRD-P2P-CLAIMS-004 | CLAIMS-3/4 | `test_tier_required` | engineering blocker handoff 与下轮复评触发条件冻结 | 后续工程推进方向 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-P2P-CLAIMS-001 | 在 `MAINNET-1~3` 之后单独做一次 public claims re-evaluation | 默认把三个 gate 完成视作整体准备就绪 | 三个 gate 当前都只是规格冻结，不是工程闭环。 |
| DEC-P2P-CLAIMS-002 | 当前对外口径上限固定为 `limited playable technical preview` + `crypto-hardened preview` | 使用“接近 mainnet-grade”或“准主链级”模糊描述 | 模糊描述会再次高估当前安全成熟度。 |
| DEC-P2P-CLAIMS-003 | 未来升级 claim 必须绑定 execution blockers 清零 | 只要文档和设计写全就允许升级 | 口径升级必须以真实执行和 QA/pass 为前提。 |
