# scripts 文档索引

审计轮次: 6

## 入口
- PRD: `doc/scripts/prd.md`
- 设计总览: `doc/scripts/design.md`
- 标准执行入口: `doc/scripts/project.md`
- 文件级索引: `doc/scripts/prd.index.md`

## 模块职责
- 维护仓内高频脚本的主入口、参数契约与 fallback 围栏口径。
- 维护 worktree 级隔离 harness，让 agent / QA 能并行起栈并读取稳定状态文件。
- 维护标准化 task worktree bootstrap 与 landing 入口，让每个新需求按统一 branch/path 命名落到独立 worktree，并可选直接检查模块文档、预热 harness、标准化合入本地 `main`。
- 汇总 precommit、viewer-tools、wasm 与治理专题文档。
- 承接脚本稳定性趋势、文档门禁与运行约束收口。

## 主题文档
- `precommit/`：提交前检查与门禁策略。
- `viewer-tools/`：viewer 抓帧与纹理质检工具链路。
- `wasm/`：WASM 构建脚本与环境约束。
- `governance/`：脚本分层、参数契约、稳定性趋势、worktree harness 与 task worktree bootstrap 专题。

## 近期专题
- `doc/scripts/governance/script-entry-layering-2026-03-11.prd.md`
- `doc/scripts/governance/script-parameter-contracts-2026-03-11.prd.md`
- `doc/scripts/governance/script-stability-trend-tracking-2026-03-11.prd.md`
- `doc/scripts/governance/worktree-isolated-harness-2026-03-27.prd.md`
- `doc/scripts/governance/task-worktree-bootstrap-2026-03-27.prd.md`
- `doc/scripts/governance/task-worktree-landing-2026-03-27.prd.md`
- `doc/scripts/precommit/pre-commit.prd.md`
- `doc/scripts/viewer-tools/capture-viewer-frame.prd.md`

## 根目录收口
- 模块根目录主入口保留：`README.md`、`prd.md`、`design.md`、`project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `precommit/`、`viewer-tools/`、`wasm/`、`governance/`。

## 维护约定
- 脚本行为变化需同步更新对应文档、测试口径与参数契约说明。
- 新增专题后，需同步回写 `doc/scripts/prd.index.md` 与本目录索引。
- `scripts/new-task-worktree.sh` 为新需求默认入口；`--init-docs` 用于检查模块 PRD / project / 当日 devlog，`--with-harness` 用于在新 worktree 中后台预热 `./scripts/worktree-harness.sh up --no-llm`。
- `scripts/land-task-worktree.sh` 为任务完成后的默认 landing 入口；负责在干净 task worktree 与干净本地 `main` worktree 之间执行标准化 rebase + fast-forward 合入，并输出回收建议。
