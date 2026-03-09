# world-simulator 文档索引

审计轮次: 6

## 入口
- PRD: `doc/world-simulator/prd.md`
- 设计总览: `doc/world-simulator/design.md`
- 标准执行入口: `doc/world-simulator/project.md`
- 兼容执行入口: `doc/world-simulator/project.md`
- 文件级索引: doc/world-simulator/prd.index.md

## 主题目录
- `viewer/`: Viewer 与 Web/交互/可视化相关设计。
- `llm/`: LLM 行为、Prompt、评估与稳定性相关设计。
- `launcher/`: 启动器与链路编排相关设计。
- `scenario/`: 场景定义、初始化与配置相关设计。
- `kernel/`: 内核规则桥接与 WASM 规则执行相关设计。
- `m4/`: M4 专题文档。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`project.md`、`prd.index.md`。
- 其余专题文档已迁移到对应主题目录（`viewer/llm/launcher/scenario/kernel/m4`）。

## 专项手册
- Viewer 使用手册：`doc/world-simulator/viewer/viewer-manual.md`

## 根目录 legacy
- `doc/world-simulator.prd.md`
- `doc/world-simulator.project.md`

## 维护约定
- 新文档按主题目录落位，不再默认平铺在模块根目录。
- 模块行为变更需同步更新 `prd.md` 与 `project.md`。
