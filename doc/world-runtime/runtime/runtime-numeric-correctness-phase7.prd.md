# Agent World Runtime：PoS 超多数比率边界数值语义硬化（15 点清单第七阶段）

审计轮次: 4

- 对应项目管理文档: doc/world-runtime/runtime/runtime-numeric-correctness-phase7.prd.project.md

## 1. Executive Summary
- 收口 PoS 配置校验中 `supermajority_numerator * 2` 的饱和乘法语义，避免极端大整数比率下出现“本应合法却被误拒绝”的隐性数值失真。
- 统一 `proto/consensus/node` 三处超多数比率判定逻辑，保证同一输入在各层行为一致。
- 补齐大整数边界测试，确保长期运行系统在极端参数下仍具确定性语义。

## 2. User Experience & Functionality
### In Scope（第七阶段）
- `agent_world_proto::distributed_pos::required_supermajority_stake`：
  - 将 `numerator.saturating_mul(2) <= denominator` 改为无乘法溢出的判定方式（等价于 `numerator <= denominator / 2`）。
- `agent_world_consensus::PosConsensus::new`：
  - 同步采用无溢出超多数比率判定。
- `agent_world_node::pos_validation::validated_pos_state`：
  - 同步采用无溢出超多数比率判定。
- 测试：
  - 新增/更新大整数边界测试，覆盖 `denominator = u64::MAX`、`numerator = denominator / 2 + 1` 等场景。

### Out of Scope（后续阶段）
- PoS 其他统计性 `saturating_*`（如 epoch 回溯辅助语义）的大范围语义重构。
- 跨模块 stake 类型升级（如 newtype/U256）与链上编码兼容变更。
- 非 PoS 主路径的通用计数器饱和语义清理。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- 不改变公开结构体与函数签名：
  - `required_supermajority_stake(total_stake, numerator, denominator) -> Result<u64, String>`
  - `PosConsensus::new(config) -> Result<PosConsensus, WorldError>`
  - `validated_pos_state(...) -> Result<...>`
- 仅调整“> 1/2”判断实现方式，保证在超大整数输入下语义与数学定义一致。

## 5. Risks & Roadmap
- M0：Phase7 建档并冻结范围。
- M1：proto/consensus/node 三处比率判定逻辑完成一致化改造。
- M2：边界测试通过并完成回归。
- M3：文档/devlog 收口，阶段完成。

### Technical Risks
- 比率判定方式切换可能影响既有极端参数用例预期，需要同步更新测试断言。
- 若上层配置系统依赖旧的“误拒绝”行为，切换后会改变可接受配置集合。
- 需避免仅改单层导致跨层语义分裂（proto 与 node/consensus 不一致）。

## 当前状态
- 截至 2026-02-23：M0、M1、M2、M3 全部完成。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-ENGINEERING-006 | 文档内既有任务条目 | `test_tier_required` | `./scripts/doc-governance-check.sh` + 引用可达性扫描 | 迁移文档命名一致性与可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-DOC-MIG-20260303 | 逐篇阅读后人工重写为 `.prd` 命名 | 仅批量重命名 | 保证语义保真与审计可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章 Executive Summary。
- 原“范围” -> 第 2 章 User Experience & Functionality。
- 原“接口 / 数据” -> 第 4 章 Technical Specifications。
- 原“里程碑/风险” -> 第 5 章 Risks & Roadmap。
