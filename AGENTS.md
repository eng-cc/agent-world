## 开发工作流
1. 拿到新需求时先阅读PRD 文档、项目管理文档，了解现状(有时开发日志也可以作为补充性资料)
  1.1 每个模块固定维护一个 PRD 主文档: `doc/<module>/prd.md`；书写PRD需要使用prd skill(.agents/skills/prd)
  1.2 doc下面的PRD文档通过引用关系组成一棵大树，能通过这个文档树承载项目的全貌
  1.3 每个PRD文档维护一个主项目管理文档: `doc/<module>/prd.project.md`；`prd.project.md` 过长时可拆分为 `doc/<module>/prd.project/*.md`（可多层嵌套），并由主文档统一索引与汇总；项目管理文档至少包含:任务拆解(含 PRD-ID 映射)、依赖、状态（可简要）
  1.4 PRD / project / devlog 分工（原则）：
    - PRD（`doc/<module>/prd.md`、`doc/**/**.prd.md`）：只写目标态规格（Why/What/Done）。
    - 项目管理（`doc/**/**.prd.project.md`）：只写执行计划（How/When/Who）。
    - 任务日志（`doc/devlog/YYYY-MM-DD.md`）：只写当天过程（时刻、完成内容、遗留事项），后续不改。
    - 详细写作约束与审查门禁以 `.agents/skills/prd/SKILL.md` 与 `.agents/skills/prd/check.md` 为准。
2. 必须先创建/补齐/修改对应模块的 `prd.md`，再根据 PRD 拆解 `prd.project.md`(拆成具体任务),作为一个commit提交
3. 根据拆解好的任务写代码、跑测试，每次完成一个任务
  3.1 所有代码和功能(包括UI)应该都是可以被测试的，单元测试或者模拟闭环测试都可以
  3.2 所有测试都分 `test_tier_required` 或 `test_tier_full`
  3.3 系统性测试流程与套件矩阵统一参考 `testing-manual.md`
4. 每次改完代码必须回顾并更新对应模块的 `prd.md` 与 `prd.project.md`，保证代码/测试可追溯到 PRD-ID；模块需求或行为改动时必须同步更新 `prd.md`
5. 检查单个 rust 代码文件不能超过 1200 行，如果超限可适当拆分；
6. 每个任务(指项目文档里拆解的任务)完成后都需要写任务日志，跑测试
  6.1 任务日志写`doc/devlog` 文件夹下,文件名为年-月-日.md
  6.2 任务日志应包含:时刻(需要每次任务确认下当前时刻)、完成内容、遗留事项；日志需要保障符合当天实际，后续不能修改，没有行数限制；每天一个日志文件，无需拆分
7. 每个任务(写文档也算一个任务)一个 commit，无需询问，及时提交
8. 每个需求，只要当前项目管理文档中还有后续任务就不要中断，开始下一个任务即可

## 工程架构
- 各个子模块各自闭环基础模块功能
- third_party下面的代码只可读，不能写
- 执行cargo命令需要如下形式 env -u RUSTC_WRAPPER cargo check
- 使用手册都放在site/doc(cn/en)，可作为静态站内容

## Agent 专用：UI Web 闭环调试（给 Codex 用，Playwright 优先）
- 目标与完整流程已迁移至 `testing-manual.md`（S6 及其补充约定）。
- 约束保持不变：
  - Web 闭环为默认链路（Playwright 优先）。
  - `capture-viewer-frame.sh` 仅在 native 图形链路问题或 Web 无法复现时使用。

# Project Agents

See `third_party/rust-skills/AGENTS.md` for Rust development guidelines.

## 分工
根 `AGENTS.md` 只维护组合角色入口；详细职责、输入输出、决策边界与完成定义统一写在 `.agents/roles/*.md`。

1. `producer_system_designer`
   - 入口：`.agents/roles/producer_system_designer.md`
   - 覆盖：制作人 / 产品负责人、世界规则策划、涌现系统策划、经济 / 资源策划

2. `runtime_engineer`
   - 入口：`.agents/roles/runtime_engineer.md`
   - 覆盖：运行时 / 世界内核工程师、仿真 / 数值平衡工程师

3. `wasm_platform_engineer`
   - 入口：`.agents/roles/wasm_platform_engineer.md`
   - 覆盖：WASM 平台 / 模块生态工程师

4. `agent_engineer`
   - 入口：`.agents/roles/agent_engineer.md`
   - 覆盖：Agent 行为设计师、AI / Agent 工程师

5. `viewer_engineer`
   - 入口：`.agents/roles/viewer_engineer.md`
   - 覆盖：前端 / Viewer / 交互设计师

6. `qa_engineer`
   - 入口：`.agents/roles/qa_engineer.md`
   - 覆盖：测试 / 自动化 / 世界 QA

7. `liveops_community`
   - 入口：`.agents/roles/liveops_community.md`
   - 覆盖：运营 / 社区 / 世界管理员

### 使用约定
- 新需求优先在对应角色职责卡中确认 owner、输入、输出与 done 定义；如跨多个角色，按最先落地代码/文档的 owner 牵头。
- 根 `AGENTS.md` 不再扩写角色细节；角色职责调整时，直接修改 `.agents/roles/*.md`，必要时同步回写 engineering `prd.md` / `prd.project.md`。
- 角色职责卡用于人机协作对齐，不替代模块 `prd.md` / `prd.project.md` 的需求与任务追踪。
