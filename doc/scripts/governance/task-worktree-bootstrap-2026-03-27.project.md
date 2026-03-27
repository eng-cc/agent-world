# oasis7: task worktree bootstrap 标准入口（2026-03-27）（项目管理）

- 对应设计文档: `doc/scripts/governance/task-worktree-bootstrap-2026-03-27.design.md`
- 对应需求文档: `doc/scripts/governance/task-worktree-bootstrap-2026-03-27.prd.md`

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] WTB-BOOT-1 (PRD-SCRIPTS-WTB-001/002) [test_tier_required]: 新增 `scripts/new-task-worktree.sh`，实现默认 branch/path 派生、clean-source guard 与 branch/path 冲突检测。
- [x] WTB-BOOT-2 (PRD-SCRIPTS-WTB-001/002/003) [test_tier_required]: 为入口补齐 `--base`、`--branch`、`--path`、`--json`、`--allow-dirty-source` 契约与人类可读下一步提示。
- [x] WTB-BOOT-3 (PRD-SCRIPTS-WTB-003) [test_tier_required]: 同步 `AGENTS.md`、`doc/scripts/{prd,project,README,prd.index}.md` 与 `doc/devlog/2026-03-27.md`，把新标准入口收入口径。
- [x] WTB-BOOT-4 (PRD-SCRIPTS-WTB-004) [test_tier_required]: 为入口补齐 `--init-docs` 与 `--with-harness`，输出模块文档检查摘要，并可在新 worktree 中后台预热 `worktree-harness.sh up --no-llm`。

## 关键契约

### 1. 默认命名
| 字段 | 默认值 |
| --- | --- |
| branch | `task/<module>-<task>` |
| worktrees_root | `<repo-parent>/worktrees` |
| worktree_path | `<worktrees_root>/<repo-name>-<module>-<task>` |
| base_ref | `HEAD` |

### 2. 输出字段
| 字段 | 含义 |
| --- | --- |
| `module` | 原始 module 输入 |
| `task` | 原始 task 输入 |
| `repo_name` | worktree family 使用的稳定 repo 名称 |
| `worktrees_root` | worktree family 使用的默认根目录 |
| `branch` | 最终使用的 branch |
| `worktree_path` | 最终 worktree 路径 |
| `base_ref` | 创建或附着所基于的 ref |
| `mode` | `create_new_branch` 或 `attach_existing_branch` |
| `doc_checks` | `--init-docs` 时的模块 PRD / project / 当日 devlog 检查结果 |
| `harness` | `--with-harness` 时的 bootstrap 日志、state 文件、状态与 viewer URL 摘要 |

## 依赖
- `AGENTS.md`
- `scripts/new-task-worktree.sh`
- `doc/scripts/project.md`

## 状态
- 更新日期：2026-03-27
- 当前阶段：已完成
- 阻塞项：无
- 下一步：若后续对命名模板还有更强约束，可再补一轮 task type / owner / date 维度的命名治理专题；若要继续减操作数，可再评估 docs skeleton/scaffold 是否值得引入。
