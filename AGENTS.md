## 开发工作流
1. 拿到仓库时先阅读开发日志、PRD 文档、项目管理文档，了解现状
  1.1 每个模块固定维护一个 PRD 主文档: `doc/<module>/prd.md`；书写PRD需要使用prd skill(.agents/skills/prd)
  1.2 doc下面的PRD文档通过引用关系组成一棵大树，能通过这个文档树承载项目的全貌
  1.3 每个PRD文档维护一个主项目管理文档: `doc/<module>/prd.project.md`；`prd.project.md` 过长时可拆分为 `doc/<module>/prd.project/*.md`（可多层嵌套），并由主文档统一索引与汇总；项目管理文档至少包含:任务拆解(含 PRD-ID 映射)、依赖、状态（可简要）
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
