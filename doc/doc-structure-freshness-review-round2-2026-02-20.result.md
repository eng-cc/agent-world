# Doc 目录结构与内容时效复核（Round 2）结果文档

## 复核范围
- 全量文档：`doc/**/*.md`
- 覆盖数量：`529`（`active=394`、`archive=117`、`devlog=18`）
- 关联实现核对目录：
  - `crates/`
  - `scripts/`
  - `site/`
  - `tools/`

## 复核方法
- 全量机器读取（逐文件）：
  - 读取每个文档正文并生成清单：`output/doc-structure-freshness-review-round2-2026-02-20.json`
  - 校验 markdown 链接有效性与文档路径引用一致性
  - 统计目录分布、活跃/归档/日志分层占比、超行数文件
- 人工复核（重点样本）：
  - 逐篇复核阶段性 LLM 文档及其后续替代文档：
    - `doc/world-simulator/archive/llm-build-chain-actions.md`
    - `doc/world-simulator/archive/llm-build-chain-actions.project.md`
    - `doc/world-simulator/archive/llm-factory-actions.md`
    - `doc/world-simulator/archive/llm-factory-actions.project.md`
    - `doc/world-simulator/llm-factory-strategy-optimization.md`
    - `doc/world-simulator/llm-factory-strategy-optimization.project.md`
    - `doc/world-simulator/llm-industrial-mining-debug-tools.md`
    - `doc/world-simulator/llm-industrial-mining-debug-tools.project.md`
  - 对照当前实现：
    - `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
    - `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
    - `crates/agent_world/src/simulator/kernel/actions.rs`

## 目录结构结论
- 结论：当前 `doc/` 分层总体合理，可继续使用。
- 依据：
  - 顶层已收敛为入口型文档（总览/索引/专项审计），主题内容主要下沉到子目录。
  - 归档目录按主题隔离（`*/archive/`），活跃文档与历史文档边界清晰。
  - `doc/README.md` 已能覆盖主要导航需求。
- 剩余建议（非阻塞）：
  - `doc/world-simulator/` 体量仍最大，后续可在不破坏链接前提下进一步分组（如 `llm/`、`viewer/`、`scenario/`）。

## 内容时效结论
- 结论：大部分活跃文档仍可作为当前基线，仅确认 2 组早期阶段文档过时并归档。
- 全量扫描结论：
  - 活跃文档 markdown 断链：`0`
  - 活跃文档高风险引用：集中在阶段性历史产物路径（`output/*`），不影响当前代码实现口径
- 新增归档（本轮）：
  - `doc/world-simulator/archive/llm-build-chain-actions.md`
  - `doc/world-simulator/archive/llm-build-chain-actions.project.md`
  - `doc/world-simulator/archive/llm-factory-actions.md`
  - `doc/world-simulator/archive/llm-factory-actions.project.md`
- 归档理由：
  - 上述文档覆盖的是早期“动作接入/建造链路”阶段目标，已被后续 LFO/MMD 文档与当前实现替代。
  - 文档中“Out of Scope”与当前实现状态存在阶段性偏差（例如后续已完成更深层闭环与 guardrail 收敛）。

## 同步修正
- 引用修复：
  - `doc/world-simulator.project.md` 中 `llm-build-chain-actions*` 链接改为归档路径。
- 规范修正：
  - `doc/p2p/archive/distributed-crate-split-net-consensus.md` 行数从 `501` 调整为 `500`（满足单文档上限约束）。

## 最终判定
- 目录结构：`合理（通过）`
- 内容时效：`总体有效，局部阶段文档已归档（通过）`
- 后续维护策略：
  - 新阶段文档收口后，优先归档“阶段完成且被后续基线替代”的文档。
  - 保持 `doc/devlog` 历史快照属性，不做追溯改写。
