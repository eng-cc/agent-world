## 开发工作流（PRD硬切换）
0. 生效日期与范围: 自 2026-03-02 起，新需求与进行中需求统一走 PRD 流程；旧“设计文档入口”停止新增，历史文档仅作 `legacy` 参考。
1. 拿到仓库时先阅读开发日志、PRD 文档、项目管理文档，了解现状
  1.1 每个模块固定维护一个 PRD 主文档: `doc/<module>/prd.md`（同一模块不再创建并行 PRD 命名变体）；doc下面的文档通过引用关系组成一棵大树
  1.2 每个模块固定维护一个项目管理文档: `doc/<module>/prd.project.md`
  1.3 PRD 文档至少包含:Executive Summary(Problem Statement、Proposed Solution、Success Criteria)、User Experience & Functionality(User Personas、User Stories、Acceptance Criteria、Non-Goals)、AI System Requirements(如适用)、Technical Specifications、Risks & Roadmap
  1.4 项目管理文档至少包含:任务拆解(含 PRD-ID 映射)、依赖、状态（可简要）
2. 新功能和在研需求必须先创建/补齐对应模块的 `prd.md`，再根据 PRD 拆解 `prd.project.md`(拆成具体任务)
3. 根据拆解好的任务写代码、跑测试，每次完成一个任务
  3.1 所有代码和功能(包括UI)应该都是可以被测试的，单元测试或者模拟闭环测试都可以
  3.2 所有测试都分 `test_tier_required` 或 `test_tier_full`
  3.3 系统性测试流程与套件矩阵统一参考 `testing-manual.md`
4. 每次改完代码必须回顾并更新对应模块的 `prd.md` 与 `prd.project.md`，保证代码/测试可追溯到 PRD-ID；模块需求或行为改动时必须同步更新 `prd.md`
5. 检查单个 rust 代码文件不能超过 1200 行，单个 md 文档不能超过 500 行；`prd.md` 超过 500 行时拆分为 `doc/<module>/prd/*.md`(可以嵌套多层文件夹)，并保留 `doc/<module>/prd.md` 作为总览入口
6. 每个任务(指项目文档里拆解的任务)完成后都需要写任务日志，跑测试
  6.1 任务日志写在当前设计涉及的最外层文件夹的 `doc/devlog` 文件夹下,文件名为年-月-日.md
  6.2 任务日志应包含:时刻(需要每次任务确认下当前时刻)、完成内容、遗留事项；日志需要保障符合当天实际，后续不能修改，没有行数限制；每天一个日志文件，无需拆分
7. 每个任务(写文档也算一个任务)一个 commit，无需询问，及时提交
8. 只要当前项目管理文档中还有后续任务就不要中断，开始下一个任务即可

## 工程架构
- 本仓库内所有新功能必须包含:`doc/<module>/prd.md`、`doc/<module>/prd.project.md`、代码
- 主crate是agent_world其他子模块各自闭环基础模块功能
- third_party下面的代码只可读，不能写
- 执行cargo命令需要如下形式 env -u RUSTC_WRAPPER cargo check
- 使用手册都放在site/doc(cn/en)，可作为静态站内容

## Agent 专用：UI Web 闭环调试（给 Codex 用，Playwright 优先）
- 目标与完整流程已迁移至 `testing-manual.md`（S6 及其补充约定）。
- 约束保持不变：
  - Web 闭环为默认链路（Playwright 优先）。
  - `capture-viewer-frame.sh` 仅在 native 图形链路问题或 Web 无法复现时使用。
