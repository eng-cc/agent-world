# scripts 文档索引

审计轮次: 6

## 入口
- PRD: `doc/scripts/prd.md`
- 设计总览: `doc/scripts/design.md`
- 标准执行入口: `doc/scripts/project.md`
- 兼容执行入口: `doc/scripts/project.md`
- 文件级索引: doc/scripts/prd.index.md

## 主题文档
- `precommit/`：提交前检查与门禁策略。
- `viewer-tools/`：viewer 抓帧与纹理质检工具链路。
- `wasm/`：WASM 构建脚本与环境约束。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `precommit/viewer-tools/wasm`。

## 维护约定
- 脚本行为变化需同步更新对应文档与测试口径。
- `doc/scripts/precommit/pre-commit.design.md`
- `doc/scripts/precommit/precommit-remediation-playbook.design.md`
- `doc/scripts/viewer-tools/capture-viewer-frame.design.md`
- `doc/scripts/viewer-tools/viewer-texture-inspector-art-capture-2026-02-28.design.md`
- `doc/scripts/viewer-tools/viewer-texture-inspector-framework-rationalization-2026-02-28.design.md`
- `doc/scripts/viewer-tools/viewer-texture-inspector-framework-rationalization-2026-03-01.design.md`
- `doc/scripts/viewer-tools/viewer-texture-inspector-material-recognizability-2026-02-28.design.md`
- `doc/scripts/viewer-tools/viewer-texture-inspector-visual-detail-system-optimization-2026-02-28.design.md`
- `doc/scripts/wasm/builtin-wasm-nightly-build-std.design.md`
