# Doc 文档归档审计结果（2026-02-20）

## 审计范围
- 全量扫描：`doc/**/*.md`
- 排除说明：仅 `doc/devlog/**` 作为历史日志保留，不参与归档迁移判定
- 对照对象：`doc/` 交叉引用、`crates/`、`scripts/`、`site/`、`tools/` 当前实现

## 审计方法
- 机器审计（全覆盖）：
  - 提取每篇文档中的路径引用，校验是否存在失效链接。
  - 统计文档间引用关系（入度/出度）与基础状态（`archive` / `devlog` / `project`）。
  - 输出全量清单：`output/doc-archive-audit-2026-02-20.json`。
- 人工复核（重点）：
  - 对“迁移阶段文档、发布清单、旧路径高风险文档”逐篇阅读。
  - 结合同主题后续文档与当前代码实现，确认是否归档。

## 全量结果
- 总文档数：527
- 非 devlog 文档：509
- 已在 archive（含本轮新增）：117
- 机器审计复核待处理：0

## 本轮新增归档

### world-runtime（迁移阶段已收口，保留历史追溯）
- `doc/world-runtime/archive/builtin-wasm-crate-split.md`
- `doc/world-runtime/archive/builtin-wasm-crate-split.project.md`
- `doc/world-runtime/archive/builtin-wasm-independent-module-crates.md`
- `doc/world-runtime/archive/builtin-wasm-independent-module-crates.project.md`
- `doc/world-runtime/archive/builtin-wasm-lifecycle-sdk.md`
- `doc/world-runtime/archive/builtin-wasm-lifecycle-sdk.project.md`
- `doc/world-runtime/archive/builtin-wasm-runtime-core-replacement.md`
- `doc/world-runtime/archive/builtin-wasm-runtime-core-replacement.project.md`
- `doc/world-runtime/archive/builtin-wasm-module-business-logic-migration.md`
- `doc/world-runtime/archive/builtin-wasm-module-business-logic-migration.project.md`

### site（发布阶段文档，已被后续站点现状替代）
- `doc/site/archive/github-pages-wow-polish.md`
- `doc/site/archive/github-pages-wow-polish.project.md`
- `doc/site/archive/github-pages-wow-polish.release-checklist.md`

### p2p（迁移映射/阶段审计历史记录）
- `doc/p2p/archive/migration-map.md`
- `doc/p2p/archive/staleness-audit-2026-02-16.md`

### scripts（已终止路线归档）
- `doc/scripts/archive/builtin-wasm-canonical-build-environment.md`
- `doc/scripts/archive/builtin-wasm-canonical-build-environment.project.md`

## 保留并修正引用
- `doc/world-runtime.md`：修复分册索引到 `doc/world-runtime/archive/*`。
- `doc/world-runtime.project.md`：修复 WRS/R2~R8 与 BMS 文档链接到归档路径。
- `doc/p2p/p2p-doc-consolidation.md`：迁移清单链接改为 `doc/p2p/archive/migration-map.md`。
- `doc/site/site-manual-static-docs.project.md`：任务日志路径统一为 `doc/devlog/2026-02-15.md`。
- `doc/site/viewer-manual-content-migration-2026-02-15.project.md`：任务日志路径统一为 `doc/devlog/2026-02-15.md`。
- `doc/world-simulator/kernel-rule-wasm-sandbox-bridge.project.md`：依赖路径改为 `crates/agent_world_wasm_executor/src/lib.rs`。
- `doc/world-simulator/rust-wasm-build-suite.project.md`：依赖路径改为 `crates/agent_world_wasm_executor/src/lib.rs`。
- `doc/world-simulator/llm-factory-strategy-optimization.md`：移除已失效测试文件路径，改为现有测试文件组织描述。
- `doc/world-simulator/llm-prompt-multi-step-orchestration.md`：将 tools/tool_calls 调整为术语描述，避免误导为仓库路径。

## 结论
- 本轮归档判定已完成，文档树中“历史迁移/发布阶段文档”已转入对应 `archive/`。
- 活跃文档的关键失效引用已修复，后续可基于该基线继续做主题级精细化治理。
