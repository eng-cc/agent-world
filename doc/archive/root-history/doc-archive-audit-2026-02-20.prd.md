# Doc 文档归档审计（2026-02-20）设计文档

## 1. Executive Summary
- 对 `doc/` 下非 `devlog`、非既有 `archive` 文档执行一次“文档-相关文档-当前代码实现”一致性审计。
- 为每篇文档给出归档判定：`保留` / `归档` / `保留但修正引用`。
- 将确认过期且已被后续文档替代的文档迁移到对应 `archive/` 目录并标记“已归档”。

## 2. User Experience & Functionality
### In Scope
- 审计对象：
  - `doc/**/*.md`（排除 `doc/devlog/**` 与现有 `*/archive/**`）。
- 判定维度：
  - 是否引用已删除代码路径或已失效脚本入口。
  - 是否被同主题后续文档明确替代（里程碑收口后有新基线）。
  - 是否仅为阶段性发布/迁移过程记录，且当前不再作为实施依据。
- 落地动作：
  - 迁移确认过期文档到同主题 `archive/`。
  - 修复保留文档中的失效路径引用。
  - 输出审计结果清单（含每篇文档判定与理由）。

### Out of Scope
- 不改写 `doc/devlog` 历史日志。
- 不改动业务代码语义；仅做文档治理与引用修复。
- 不处理 `third_party/` 内容本身。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- 新增审计结果：
  - `doc/archive/root-history/doc-archive-audit-2026-02-20.result.md`（人工结论摘要）
  - `output/doc-archive-audit-2026-02-20.json`（全量清单，机器可读）
- 归档目录（按主题）：
  - `doc/world-runtime/archive/`
  - `doc/site/archive/`（若本轮出现 site 文档归档）
  - `doc/world-simulator/archive/`（若本轮出现 simulator 文档归档）
- 归档标识模板：
  - `> [!WARNING]`
  - `> 该文档已归档，仅供历史追溯，不再作为当前实现依据。`
  - `> 归档日期：2026-02-20`

## 5. Risks & Roadmap
- M1：输出审计设计与项目管理文档，冻结判定标准。
- M2：完成全量清单审计与分组判定（保留/归档/修正）。
- M3：完成确认归档文档迁移、引用修复与结果发布。

### Technical Risks
- 风险：把“仍有参考价值的闭环设计”误归档。
  - 缓解：仅归档“被明确替代且引用失效”的文档；边界不清时先保留并修正引用。
- 风险：批量迁移导致链接断裂。
  - 缓解：迁移后执行路径扫描并修复非 `devlog` 引用。

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
