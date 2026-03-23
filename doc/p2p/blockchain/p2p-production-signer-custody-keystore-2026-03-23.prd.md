# oasis7 生产级 signer custody / keystore 基线（2026-03-23）

- 对应设计文档: `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.design.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.project.md`

审计轮次: 1
## 1. Executive Summary
- Problem Statement: oasis7 已完成公开主链资产面的 signed payload gating，但当前节点 signer 仍会自动生成并明文写入 `config.toml`，Web/native signer 也仍依赖 env 或本地 `config.toml` bootstrap。只要 signer custody 仍停留在这种 preview 级路径，系统就不能进入 `mainnet-grade candidate`。
- Proposed Solution: 建立 producer-owned 的“生产级 signer custody / keystore”专题 PRD，明确哪些 signer 仍是 local bootstrap、哪些生产路径必须迁移到离线存储 + 人工多签 custody，并冻结 rotation、revocation、audit trail、人工操作边界与环境分层门禁。
- Success Criteria:
  - SC-1: 覆盖至少三类 signer scope：`node runtime signer`、`viewer/player signer bootstrap`、`governance/controller signer policy source`。
  - SC-2: 明确当前 `config.toml` 明文私钥、页面注入私钥 bootstrap 与 env 直传私钥均只允许存在于 local/dev/preview，不得作为 production path。
  - SC-3: 给出生产级 signer custody 的最小完成定义：`offline storage + manual multisig`、`rotation`、`revocation`、`audit trail`、`environment policy`，并明确 operator key staging path 为 operator-local non-repo 目录（例如 `~/Documents/keys`，非自动注入路径）。
  - SC-4: 形成可执行任务链，至少拆出 `signer inventory/failure policy/config boundary/QA gate` 四个切片。
  - SC-5: 在本专题完成前，整体安全阶段仍保持 `crypto-hardened preview`，不得升级为 `mainnet-grade`。

## 2. User Experience & Functionality
- User Personas:
  - `producer_system_designer`：需要把“现在哪些 signer 还能继续本地注入、哪些必须进受控托管”冻结成正式边界。
  - `runtime_engineer`：需要知道 node/governance/viewer 三类 signer 的生产路径该如何拆分。
  - `qa_engineer`：需要有可以阻断的 custody gate，而不是只检查“能不能签出来”。
  - `liveops_community`：需要知道发布环境是否还允许携带本地私钥 bootstrap。
  - 运维/金库维护者：需要明确谁负责生成、持有、轮换和吊销生产 signer。
- User Scenarios & Frequency:
  - 每次准备把 preview 环境升级到更高可信度时执行。
  - 每次新增 signer surface 或修改启动配置时复核。
  - 每次准备推进治理 signer 外部化或创世 ceremony 前执行。
- User Stories:
  - PRD-P2P-CUSTODY-001: As a `producer_system_designer`, I want one formal inventory of all current signer bootstrap paths, so that preview convenience is not mistaken for production custody.
  - PRD-P2P-CUSTODY-002: As a `runtime_engineer`, I want a target architecture for managed keystore / external signer boundaries, so that runtime work is sequenced correctly.
  - PRD-P2P-CUSTODY-003: As a `qa_engineer`, I want rotation/revocation/audit trail to be explicit gate conditions, so that signer custody has pass/block criteria.
  - PRD-P2P-CUSTODY-004: As a `liveops_community`, I want environment-level policy for which signer injection methods are allowed, so that production releases do not accidentally ship preview bootstrap paths.
- Critical User Flows:
  1. Flow-P2P-CUSTODY-001: `盘点 node/viewer/governance signer 来源 -> 标记 local/dev/preview/production 允许级别 -> producer 冻结 source-of-truth`
  2. Flow-P2P-CUSTODY-002: `runtime 定义 offline storage + manual multisig 边界 -> 配置层禁止明文私钥继续作为生产入口 -> key staging path 仅用于人工 custody 流程 -> QA 复核 gate`
  3. Flow-P2P-CUSTODY-003: `发生 signer compromise 或运维轮换需求 -> 按 rotation/revocation 流程切 key -> 审计日志记录 -> 恢复服务`
  4. Flow-P2P-CUSTODY-004: `liveops 准备发布 -> 检查当前环境 policy -> 若仍存在 preview bootstrap path 则阻断`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算/判定规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| Signer inventory | `signer_scope/current_source/storage_form/env_level/verdict` | 统一盘点 node/viewer/governance signer 来源 | `unknown -> inventoried -> classified` | 明文私钥、页面注入私钥、env 直传私钥只能判为 preview | `producer_system_designer` 收口 |
| Managed custody target | `signer_scope/target_backend/sign_api/key_visibility/verdict` | 为每类 signer 定义生产目标态 | `classified -> target_defined` | 生产目标态必须满足私钥不落仓、不直注入页面、不长期留明文配置；当前选定后端为 `offline storage + manual multisig` | `runtime_engineer` 牵头 |
| Rotation policy | `rotation_owner/trigger/max_overlap/evidence_id` | 定义换 key、过渡窗口和回滚规则 | `undefined -> planned -> gated` | 无 owner 或无 trigger 的 rotation 视为未完成 | producer/runtime 联审 |
| Revocation policy | `revocation_trigger/disable_path/recovery_path` | 定义 key compromise、人员变更、设备丢失时的吊销路径 | `undefined -> planned -> gated` | 无法在 release gate 中快速停用旧 signer 即判 block | `runtime_engineer` 牵头 |
| Audit trail | `signer_scope/event_type/evidence_sink/retention_class` | 记录签名请求、轮换、吊销与失败事件 | `undefined -> instrumented -> auditable` | 无审计落点或无法回放 operator 行为即不通过 | `qa_engineer` 联审 |
| Environment policy | `environment/allowed_signer_sources/reject_reason` | 冻结 local/dev/preview/production 允许的 signer 注入方式 | `draft -> enforced` | production 一律禁止 `config.toml` 明文私钥、HTML 私钥注入、env 直传私钥；operator-local key staging/export root（例如 `~/Documents/keys`）仅作为人工 custody 路径，不得被 runtime 自动读取 | `liveops_community` 执行，producer 审批 |
- Acceptance Criteria:
  - AC-1: 专题必须明确列出当前三类真实 signer 路径：`node.private_key in config.toml`、`viewer auth env/config bootstrap`、`governance/controller signer policy still local-config-derived`。
  - AC-2: 必须明确这些路径当前只允许存在于 `local/dev/preview`，不得作为 production signer custody 完成态。
  - AC-3: 必须为 `node runtime signer`、`viewer/player signer`、`governance/controller signer` 分别写出目标后端与边界；当前统一选型为 `offline storage + manual multisig`。
  - AC-4: 必须定义 `rotation`、`revocation`、`audit trail`、`environment policy` 四类 gate；任一缺失则 `MAINNET-1` 不通过。
  - AC-5: 必须明确生产环境禁止把私钥明文写进仓库、`config.toml`、HTML bootstrap、长期环境变量；operator-local key staging root（例如 `~/Documents/keys`）不得被误写成“在线生产 keystore”，只能作为人工 custody 的非仓库文件落点。
  - AC-6: 必须明确 preview 环境如果继续允许 bootstrap signer，也只能作为测试便利入口，不能提升安全等级。
  - AC-7: 必须输出 `CUSTODY-1~4` 任务链与 owner/test tier 映射。
  - AC-8: 模块主 PRD/project/index/README 与 readiness project 必须接入本专题。
- Non-Goals:
  - 本轮不直接实现 HSM/KMS、外部钱包或 signer service。
  - 本轮不完成治理 finality signer 外部化细节，那属于 `MAINNET-2`。
  - 本轮不定义创世 ceremony 执行清单，那属于 `MAINNET-3`。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 当前 signer custody 需要拆成三层看待。`node runtime signer` 仍由 `node_keypair_config` 自动生成并写入本地配置；`viewer/player signer` 仍由 `oasis7_web_launcher` 将 env/config 中的私钥 bootstrap 注入 HTML；`governance/controller signer` 虽已具备 threshold allowlist enforcement，但真值仍停留在本地配置与 deterministic/local 路径。当前 producer 决策已选定生产目标为 `offline storage + manual multisig`，并要求生产 key 只允许暂存于 operator-local 的非仓库 staging/export 根目录（例如 `~/Documents/keys`）；`MAINNET-1` 的目标不是立刻替换所有实现，而是先冻结这一 custody 边界与环境门禁。
- Integration Points:
  - `crates/oasis7/src/bin/oasis7_chain_runtime/node_keypair_config.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher/viewer_auth_bootstrap.rs`
  - `crates/oasis7/src/runtime/world/governance.rs`
  - `crates/oasis7_node/src/types.rs`
  - `crates/oasis7_node/src/node_runtime_core.rs`
  - `doc/p2p/blockchain/p2p-mainnet-grade-readiness-hardening-2026-03-23.prd.md`
  - `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 若 node key 缺失且运行时自动生成新 key 写回 `config.toml`，只能记作 preview convenience，不得作为 production bootstrap 策略。
  - 若 viewer signer 通过 HTML 注入私钥，即使签名流程正确，也只能视为 preview/test hook，不得进入 production。
  - 若治理/controller signer 只做本地 allowlist 而无外部托管、rotation 或 revocation，则 custody gate 只能给 `partial/block`。
  - 若仅把私钥文件放在 operator-local staging root（例如 `~/Documents/keys`）但仍长期驻留联网主机、被应用自动读取或未纳入人工多签审批链，则不能视为真正满足离线 custody。
  - 若环境策略未明确 production 禁止的 signer 源，任何 release 环境都必须按 `block` 处理。
  - 若 rotation 方案要求停机或无法回滚，也必须标记为未达标并留在 `planned`。
- Non-Functional Requirements:
  - NFR-P2P-CUSTODY-1: custody gate 必须区分 `local/dev/preview/production` 四级环境，不允许环境边界模糊。
  - NFR-P2P-CUSTODY-2: 生产路径的 signer 私钥不得以明文形式存在于仓库、HTML 页面、长期配置文件或长期环境变量；operator-local key staging root（例如 `~/Documents/keys`）仅允许作为 operator 控制下的非仓库文件根目录。
  - NFR-P2P-CUSTODY-3: 每类生产 signer 必须定义 rotation owner、revocation trigger 与最小审计证据。
  - NFR-P2P-CUSTODY-4: 在本专题完成前，公开安全口径仍保持 `crypto-hardened preview`。
- Security & Privacy: 本专题只允许记录 signer 范围、后端类型、公钥引用、轮换/吊销规则与审计要求；禁止在文档、日志或测试证据中落任何真实私钥、seed 或助记词。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: 完成 signer inventory、生产目标态与环境策略冻结。
  - v1.1: 定义 rotation/revocation/audit trail gate。
  - v1.2: readiness project 标记 `MAINNET-1` 完成，并把后续工作交给 `MAINNET-2`。
- Technical Risks:
  - 风险-1: 如果不先冻结 environment policy，preview bootstrap 很容易被直接带入高环境。
  - 风险-2: 如果先做 governance externalization 而 custody 边界未定，后续实现会重复返工。
  - 风险-3: 如果 rotation/revocation 只停留在口头约定，生产故障时无正式恢复路径。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-CUSTODY-001 | CUSTODY-0/1 | `test_tier_required` | 代码真值盘点、signer inventory、模块主追踪回写 | signer 来源与环境边界 |
| PRD-P2P-CUSTODY-002 | CUSTODY-1/2 | `test_tier_required` | 目标后端、source boundary、生产禁用路径与任务链冻结 | runtime/viewer/governance signer 生产路径 |
| PRD-P2P-CUSTODY-003 | CUSTODY-2/3 | `test_tier_required` | rotation/revocation/audit trail gate 定义与 QA 审核清单 | signer 事件恢复能力 |
| PRD-P2P-CUSTODY-004 | CUSTODY-3/4 | `test_tier_required` | environment policy、release gate 用语与 public claims 依赖链核对 | liveops 发布与环境治理 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-P2P-CUSTODY-001 | 先冻结 signer custody 边界，再做治理 signer 外部化 | 直接跳到 governance signer externalization | 没有 custody 边界，后续治理 signer 目标态也无法统一。 |
| DEC-P2P-CUSTODY-002 | 将 HTML/env/config 私钥注入统一归为 preview-only bootstrap | 把其中某一条继续当作生产临时方案接受 | 任何把私钥暴露给页面或长期配置的方案都不符合生产级托管。 |
| DEC-P2P-CUSTODY-003 | 生产 custody 选定为 `offline storage + manual multisig`，key staging root 定为 operator-local non-repo 目录（例如 `~/Documents/keys`） | 改走云 KMS/HSM 或自建 signer service | 当前团队优先要拿到可控、少依赖、可人工审计的执行路径，再决定后续是否引入更高自动化后端。 |
