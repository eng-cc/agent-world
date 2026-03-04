# Core PRD-ID 到测试证据映射模板（草案）

## 使用说明
- 为 `TASK-CORE-004` 提供统一映射格式。
- 每个 PRD-ID 至少对应一个任务与一组可复现证据。

## 模板字段
| PRD-ID | 任务ID | 测试层级 | 命令 | 证据路径 | 结论 |
| --- | --- | --- | --- | --- | --- |
| PRD-XXX | TASK-XXX | test_tier_required | `env -u RUSTC_WRAPPER cargo test ...` | `output/...` | pass/fail |

## 填写约束
- `测试层级` 默认使用 `test_tier_required`（core 治理任务）。
- `命令` 必须可直接执行且可复现。
- `证据路径` 必须指向仓库内可访问文件。
