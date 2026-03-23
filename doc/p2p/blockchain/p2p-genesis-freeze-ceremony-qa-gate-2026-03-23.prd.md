# oasis7 创世 freeze / ceremony / QA gate（2026-03-23）

- 对应设计文档: `doc/p2p/blockchain/p2p-genesis-freeze-ceremony-qa-gate-2026-03-23.design.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-genesis-freeze-ceremony-qa-gate-2026-03-23.project.md`

审计轮次: 1
## 1. Executive Summary
- Problem Statement: oasis7 的创世参数逻辑和分配结构已经冻结，但 `genesis freeze sheet` 仍是 `logic_frozen_address_binding_pending`，所有关键 recipient/controller slot 仍待真实绑定，signer policy 也没有形成正式 ceremony 证据 bundle。只要这些项没有进入 QA gate，项目就不能进入 `mint_ready`。
- Proposed Solution: 建立 `MAINNET-3` 专题 PRD，把创世 `recipient/controller/signer policy` 真值冻结、ceremony checklist、QA evidence bundle 和阻断条件一次性定义清楚，并明确在全部通过前 oasis7 仍然不是 `mint_ready`。
- Success Criteria:
  - SC-1: 覆盖 `slot registry freeze`、`bucket execution sheet freeze`、`signer ceremony checklist`、`QA evidence bundle` 四个核心面向。
  - SC-2: 明确 `TBD_BEFORE_MINT`、`pending_binding`、`ready_pending_address_binding` 只要任一存在，创世状态就必须保持 `not_mint_ready`。
  - SC-3: 定义 ceremony evidence bundle 的最小字段：`slot binding/public keys/controller threshold/operator checklist/qa verdict`。
  - SC-4: 形成 `GENESIS-1~4` 任务链，覆盖 freeze 真值、ceremony checklist、QA 模板和 readiness 依赖回写。
  - SC-5: 在本专题完成前，对外仍不得使用 `production mint ready` 或同义说法。

## 2. User Experience & Functionality
- User Personas:
  - `producer_system_designer`：需要把创世前最后一层“参数冻结但还未真正可执行”的灰区收成正式 gate。
  - `qa_engineer`：需要把创世放行判断从“看起来差不多”变成 checklist + evidence bundle + pass/block。
  - `runtime_engineer`：需要明确哪些 slot/recipient/controller/signer policy 在 mint 前必须成为正式真值。
  - `liveops_community`：需要知道何时才能对外说“创世准备完成”，何时绝对不能说。
  - 治理/金库维护者：需要知道创世 ceremony 前后要交哪些证据、由谁签字。
- User Scenarios & Frequency:
  - 每次准备推进创世真实地址绑定时执行。
  - 每次准备做 signer ceremony dry-run 或正式运行时执行。
  - 每次准备评估是否可以进入 mint 前复核。
- User Stories:
  - PRD-P2P-GENESIS-001: As a `producer_system_designer`, I want the genesis freeze sheet to become a hard gate, so that no unresolved slot or controller policy slips into mint execution.
  - PRD-P2P-GENESIS-002: As a `qa_engineer`, I want a formal ceremony evidence bundle and verdict template, so that mint readiness is auditable.
  - PRD-P2P-GENESIS-003: As a 治理维护者, I want a clear checklist for binding recipient/controller/signer policy truth, so that ceremony execution is unambiguous.
  - PRD-P2P-GENESIS-004: As a `liveops_community`, I want public mint-readiness claims tied to the genesis QA gate, so that the project does not overclaim.
- Critical User Flows:
  1. Flow-P2P-GENESIS-001: `读取 freeze sheet -> 检查 slot registry 与 bucket execution sheet -> 任一 TBD/pending_binding 即阻断`
  2. Flow-P2P-GENESIS-002: `绑定 recipient/controller/signer policy 真值 -> 生成 ceremony checklist -> operator 执行 -> 收集 evidence bundle`
  3. Flow-P2P-GENESIS-003: `qa_engineer 对 evidence bundle 逐项审核 -> 输出 pass/block -> producer 决定是否允许进入 mint candidate`
  4. Flow-P2P-GENESIS-004: `有人提议对外说 mint ready -> 对照 genesis QA gate -> 任一阻断项未清零则直接拒绝`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算/判定规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Slot registry freeze | `slot_id/current_value/freeze_requirement/status` | 冻结 recipient/controller slot 真值 | `pending_binding -> bound -> frozen` | 任何 `TBD_BEFORE_MINT` 或 `pending_binding` 都不得通过 | `producer_system_designer` 牵头 |
| Bucket execution freeze | `bucket_id/recipient_account_id/controller_slot_id/signer_policy/freeze_status` | 冻结每个 bucket 的 mint 前执行真值 | `ready_pending_address_binding -> bound -> frozen` | 任一 bucket 未冻结即阻断 | producer/runtime 联审 |
| Ceremony checklist | `check_id/owner/input/evidence/status` | 定义 ceremony 前、中、后检查项 | `draft -> prepared -> executed -> evidenced` | 缺 owner、缺输入或缺 evidence 都不得通过 | 治理维护者执行 |
| QA evidence bundle | `bundle_id/public_keys/controller_threshold/checklist_result/qa_verdict` | 汇总 ceremony 证据并给出结论 | `planned -> collected -> audited -> pass/block` | 无 `qa_verdict=pass` 时不得进入 mint candidate | `qa_engineer` 持有 pass/block |
| Public mint-ready gate | `claim_phrase/min_status/reject_reason` | 控制是否允许使用 mint-ready 口径 | `draft -> enforced` | 必须要求 slot/bucket/ceremony/QA 全绿 | `liveops_community` 执行，producer 审批 |
- Acceptance Criteria:
  - AC-1: 专题必须明确引用当前 freeze sheet 状态 `logic_frozen_address_binding_pending`，并把它定义为未完成创世 gate。
  - AC-2: 必须明确 `recipient_account_id = TBD_BEFORE_MINT`、`status = pending_binding`、`freeze_status = ready_pending_address_binding` 任一存在都不能进入 mint candidate。
  - AC-3: 必须定义 ceremony evidence bundle 的最小字段，且只允许记录公钥、账户绑定、threshold、审批和 QA 结论，不得记录私钥。
  - AC-4: 必须定义 `slot registry freeze`、`bucket execution freeze`、`ceremony checklist`、`QA verdict` 四类 gate；任一缺失则 `MAINNET-3` 不通过。
  - AC-5: 必须明确 `production mint ready` 口径需要最终 QA `pass`，`conditional_draft_only` 与 `block` 一律视为不通过。
  - AC-6: 必须输出 `GENESIS-1~4` 任务链与 owner/test tier 映射。
  - AC-7: 模块主 PRD/project/index/README 与 readiness project 必须接入本专题。
- Non-Goals:
  - 本轮不直接填写真实 recipient 地址、公钥或 signer 名单。
  - 本轮不直接执行 ceremony。
  - 本轮不升级整体阶段 verdict。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 当前创世执行链已经有逻辑 freeze sheet、bucket execution sheet 和 pre-mint checklist，但还缺一个把它们和 ceremony/QA 证据串起来的 producer gate。`MAINNET-3` 的目标是把 “slot/bucket 真值冻结 -> ceremony checklist -> QA evidence bundle -> mint-ready claim gate” 形成单一路径。
- Integration Points:
  - `doc/p2p/token/mainchain-token-genesis-parameter-freeze-sheet-2026-03-22.md`
  - `doc/p2p/token/mainchain-token-initial-allocation-and-early-contribution-reward-2026-03-22.prd.md`
  - `doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md`
  - `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 若逻辑参数已冻结但 slot registry 仍有 `TBD_BEFORE_MINT`，则只允许记为 `logic_frozen`，不得记为 `mint_candidate`。
  - 若 signer policy 已有 threshold 描述，但未绑定到正式 controller slot 或未形成 ceremony evidence，仍必须判 `block`。
  - 若 QA checklist 已执行但 verdict 仍为 `conditional_draft_only`，则不得进入 public mint-ready claim。
  - 若 evidence bundle 记录了私钥、seed 或敏感签名材料，则必须判失败并重做。
  - 若只完成部分 bucket 的地址绑定，则整体验证仍按失败处理，不能部分放行。
- Non-Functional Requirements:
  - NFR-P2P-GENESIS-1: 创世 gate 必须是 hard gate，不允许 `insufficient_data` 或“口头确认通过”。
  - NFR-P2P-GENESIS-2: QA evidence bundle 字段完整率必须为 `100%`，缺任一关键字段都不能给 `pass`。
  - NFR-P2P-GENESIS-3: 在本专题完成前，对外口径仍保持 `limited playable technical preview` 与 `crypto-hardened preview`。
  - NFR-P2P-GENESIS-4: 文档、日志和证据产物只允许沉淀公钥、账户绑定、threshold、checklist 和 verdict，不得落私钥、seed 或助记词。
- Security & Privacy: 本专题仅定义创世 freeze、ceremony 和 QA gate 的证据边界；任何实际 signer material 都必须在受控环境内处理，不能进入仓库或静态文档。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成 genesis freeze/ceremony/QA gate 文档与 readiness 映射。
  - v1.1: 用真实 slot binding 和 ceremony dry-run 填充 checklist 与 evidence bundle。
  - v1.2: 产出最终 QA `pass/block`，作为 `MAINNET-4` 的输入。
- Technical Risks:
  - 风险-1: 如果不把 freeze sheet 升级为硬门禁，团队可能带着 `TBD_BEFORE_MINT` 继续推进。
  - 风险-2: 如果 ceremony 没有统一 checklist，后续 evidence 无法审计。
  - 风险-3: 如果 QA verdict 不是单一事实源，mint-ready 口径会反复漂移。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-GENESIS-001 | GENESIS-0/1 | `test_tier_required` | freeze sheet 真值盘点、slot/bucket gate 定义、模块主追踪回写 | 创世参数冻结与 mint 前阻断 |
| PRD-P2P-GENESIS-002 | GENESIS-1/2/3 | `test_tier_required` + `test_tier_full` | ceremony checklist、evidence bundle 与 QA verdict 模板冻结 | 创世执行审计与放行 |
| PRD-P2P-GENESIS-003 | GENESIS-2/3 | `test_tier_required` | slot/controller/signer policy 冻结要求与 operator checklist 核对 | 治理维护与 ceremony 执行 |
| PRD-P2P-GENESIS-004 | GENESIS-3/4 | `test_tier_required` | public mint-ready gate 与 liveops 口径门禁核对 | 对外承诺与发布风险 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-P2P-GENESIS-001 | 把 freeze sheet、ceremony 与 QA 证据合并成同一个 `MAINNET-3` gate | 继续把它们分散在 token topic、聊天和 QA 口头记录里 | 创世放行需要单一事实源，不能靠多人记忆拼接。 |
| DEC-P2P-GENESIS-002 | `TBD_BEFORE_MINT/pending_binding/ready_pending_address_binding` 任一存在即 block | 允许部分 bucket 先放行 | 创世动作是整体性事件，不能局部通过。 |
| DEC-P2P-GENESIS-003 | `MAINNET-3` 先冻结证据与 checklist，不直接伪造真实地址/公钥 | 现在就把 placeholder 假装填成正式值 | 当前缺的是真值与 ceremony，不是文案数量。 |
