# Agent World: LLM 跳过 Tick 占比指标（项目管理）

## 任务拆解（含 PRD-ID 映射）
- [x] LLMSKIP-1 (PRD-TESTING-GOV-LLMSKIP-001/003): 完成专题设计文档与项目管理文档建档。
- [x] LLMSKIP-2 (PRD-TESTING-GOV-LLMSKIP-001/002): `world_llm_agent_demo` 接入 skipped tick 计数与占比输出。
- [x] LLMSKIP-3 (PRD-TESTING-GOV-LLMSKIP-002/003): `scripts/llm-longrun-stress.sh` 接入单场景与聚合指标输出。
- [x] LLMSKIP-4 (PRD-TESTING-GOV-LLMSKIP-003): 完成单测、脚本校验、devlog 与文档收口。
- [x] LLMSKIP-5 (PRD-TESTING-004): 专题文档按 strict schema 人工重写，并切换命名到 `.prd.md/.prd.project.md`。

## 依赖
- doc/testing/governance/llm-skip-tick-ratio-metric.prd.md
- `crates/agent_world/src/bin/world_llm_agent_demo.rs`
- `crates/agent_world/src/bin/world_llm_agent_demo/tests.rs`
- `scripts/llm-longrun-stress.sh`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
