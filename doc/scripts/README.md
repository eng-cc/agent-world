# scripts 文档索引

## 入口
- PRD: `doc/scripts/prd.md`
- 项目管理: `doc/scripts/prd.project.md`

## 主题文档
- `precommit/`：提交前检查与门禁策略。
- `viewer-tools/`：viewer 抓帧与纹理质检工具链路。
- `wasm/`：WASM 构建脚本与环境约束。
- `archive/`：历史脚本治理文档。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`prd.project.md`。
- 其余专题文档按主题下沉到 `precommit/viewer-tools/wasm`。

## 维护约定
- 脚本行为变化需同步更新对应文档与测试口径。
