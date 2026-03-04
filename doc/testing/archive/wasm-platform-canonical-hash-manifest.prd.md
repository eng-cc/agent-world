# Agent World：Builtin Wasm 清单收敛为“每平台 1 个 Canonical Hash”（归档专题）

> 归档说明（2026-02-20）：该方案已被 `doc/p2p/consensus/builtin-wasm-identity-consensus.prd.md` 取代，不再作为现行实现依据。

## 1. Executive Summary
- Problem Statement: 旧的 builtin wasm 清单允许任意机器 hash 累积，导致清单膨胀、hash 来源不可判定、required 门禁稳定性下降。
- Proposed Solution: 将 manifest 收敛为“每平台 1 个 canonical hash”，并改造 `sync-m1`、runtime/hydrator 与测试解析以支持 `platform=hash` 语义。
- Success Criteria:
  - SC-1: m1 manifest 统一为 `module_id <platform>=<hash> ...`。
  - SC-2: `sync-m1 --check` 仅校验当前平台 canonical hash。
  - SC-3: 同步模式仅更新当前平台 hash，不再累积任意 hash。
  - SC-4: runtime/hydrator/测试兼容 `platform=hash`，并保留 legacy token 兼容。
  - SC-5: 本归档专题迁移到 `.prd.md/.prd.project.md`，语义与替代关系完整保留。

## 2. User Experience & Functionality
- User Personas:
  - CI/测试维护者：需要可判定的 per-platform hash 校验结果。
  - Runtime/DistFS 维护者：需要稳定解析新 manifest 格式并兼容历史数据。
  - 开发者：需要在本地获得与 CI 一致的 canonical hash 校验体验。
- User Scenarios & Frequency:
  - 每次 m1 构建产物检查时执行平台 canonical 校验。
  - 跨平台开发或新增平台支持时更新允许平台集合与 hash。
  - required 门禁回归时验证 runtime/hydrator 解析兼容性。
- User Stories:
  - PRD-TESTING-ARCHIVE-CANON-001: As a test maintainer, I want one canonical hash per platform, so that manifest drift is bounded.
  - PRD-TESTING-ARCHIVE-CANON-002: As a runtime maintainer, I want parser compatibility for `platform=hash` and legacy tokens, so that upgrades are safe.
  - PRD-TESTING-ARCHIVE-CANON-003: As a developer, I want local checks to mirror CI, so that failures are reproducible.
- Critical User Flows:
  1. Flow-ARCH-CANON-001: `sync-m1 --check -> 解析当前平台键 -> 匹配 canonical hash`
  2. Flow-ARCH-CANON-002: `sync-m1 同步模式 -> 仅更新当前平台 hash -> 维持平台键唯一`
  3. Flow-ARCH-CANON-003: `runtime/hydrator 读取 manifest -> 支持 platform=hash + legacy`
  4. Flow-ARCH-CANON-004: `required 回归 -> 脚本/运行时解析一致性校验`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| manifest 规范 | `module_id <platform>=<hash> ...` | 统一写入平台键值对格式 | `legacy -> canonicalized` | 每平台仅 1 个 hash | 维护者更新 |
| 平台归一化 | `uname -s` + `uname -m` 归一化平台键 | 生成当前平台 key 用于校验/更新 | `detected -> normalized` | 不在允许集合即失败 | 脚本自动执行 |
| sync-m1 校验/同步 | `--check`、同步模式、允许平台集合 | 校验当前平台 hash 或更新当前平台槽位 | `checking/updating -> pass/fail` | 禁止追加未知平台或重复平台键 | CI/本地执行者 |
| runtime/hydrator 兼容 | `platform=hash` token + legacy token | 解析并匹配可接受 hash | `parsed -> resolved` | 优先新格式，兼容旧格式 | runtime/DistFS 维护者 |
- Acceptance Criteria:
  - AC-1: m1 manifest 完成平台 canonical 格式收敛。
  - AC-2: `sync-m1` 对平台键唯一性与允许集合做硬约束。
  - AC-3: runtime/hydrator/测试通过新旧 token 兼容回归。
  - AC-4: required 入口可稳定复现当前平台校验结果。
  - AC-5: 归档专题迁移后命名统一且替代关系保持可追溯。
- Non-Goals:
  - 不在本专题扩展 m4 到完整 canonical 数据。
  - 不更改 wasm hash 算法与 DistFS 协议。
  - 不调整 CI job 拓扑。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题聚焦 manifest 与校验链路，不涉及 AI 推理系统）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 以 `scripts/sync-m1-builtin-wasm-artifacts.sh` 为入口实现平台 canonical 校验/更新，manifest 数据模型升级为 `platform=hash`，并向 runtime/hydrator/测试层透传解析兼容。
- Integration Points:
  - `scripts/sync-m1-builtin-wasm-artifacts.sh`
  - `scripts/ci-tests.sh`
  - `crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
  - `crates/agent_world_distfs/src/bin/hydrate_builtin_wasm.rs`
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
- Edge Cases & Error Handling:
  - 当前平台不在允许集合：`sync/check` 直接失败并提示扩展配置。
  - 平台键命名漂移：校验重复平台槽位并阻断同步。
  - 旧解析链路遗漏：通过 runtime/hydrator/测试兼容改造避免加载失败。
  - legacy token 残留：保持兼容读取，逐步引导迁移。
- Non-Functional Requirements:
  - NFR-ARCH-CANON-1: per-platform 校验结论需确定且可重复。
  - NFR-ARCH-CANON-2: 清单规模受控，不随任意机器 hash 无界增长。
  - NFR-ARCH-CANON-3: required 与本地 pre-commit 校验口径一致。
- Security & Privacy: hash manifest 仅包含产物完整性摘要，不引入额外敏感信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (ARCH-CANON-1): 建立设计/项目文档并定义 canonical 目标。
  - v1.1 (ARCH-CANON-2): `sync-m1` 平台校验/同步策略落地。
  - v2.0 (ARCH-CANON-3): runtime/hydrator/测试完成 `platform=hash` 兼容。
  - v2.1 (ARCH-CANON-4): m1 清单迁移并完成 required 回归。
  - v2.2 (ARCH-CANON-5): 归档专题 strict schema 迁移与命名统一。
- Technical Risks:
  - 风险-1: 平台集合未及时扩展导致新平台阻断。
  - 风险-2: 平台键约定变化引发重复槽位或误匹配。
  - 风险-3: 旧解析逻辑遗留导致运行时加载失败。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-ARCHIVE-CANON-001 | ARCH-CANON-1/2/5 | `test_tier_required` | manifest 格式与脚本策略审阅 + 文档治理检查 | m1 清单收敛策略 |
| PRD-TESTING-ARCHIVE-CANON-002 | ARCH-CANON-2/3/4 | `test_tier_required` | runtime/hydrator 新旧 token 解析兼容回归 | 运行时加载稳定性 |
| PRD-TESTING-ARCHIVE-CANON-003 | ARCH-CANON-2/4/5 | `test_tier_required` | required 与本地校验口径一致性验证 | CI 与开发体验一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-ARCH-CANON-001 | 每平台仅 1 个 canonical hash | 任意 hash 累积 | 控制清单规模并提升可判定性。 |
| DEC-ARCH-CANON-002 | `--check` 只看当前平台 hash | 全平台一次性强校验 | 降低无关平台噪声并保持可执行性。 |
| DEC-ARCH-CANON-003 | runtime/hydrator 支持新格式并兼容 legacy | 强制一次性切断旧格式 | 降低迁移风险与回归成本。 |
| DEC-ARCH-CANON-004 | 归档后保留替代关系说明 | 删除历史专题 | 保证历史决策脉络可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标：收敛为每平台 canonical hash、避免清单膨胀、保持 gate 一致” -> 第 1 章 Summary。
- 原“In/Out Scope（格式升级、sync-m1 策略、兼容解析；不改 m4/算法/CI 拓扑）” -> 第 2 章规格矩阵与 Non-Goals。
- 原“接口/数据（manifest 行格式、平台归一化、允许集合、脚本入口）” -> 第 4 章 Integration Points。
- 原“M1~M4 里程碑” -> 第 5 章 phased rollout（ARCH-CANON-1~5）。
- 原“风险（平台集合、解析遗漏、平台键约定）” -> 第 4 章边界处理 + 第 5 章风险。
