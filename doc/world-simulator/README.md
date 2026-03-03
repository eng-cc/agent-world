# world-simulator 文档索引

## 入口
- PRD: `doc/world-simulator/prd.md`
- 项目管理: `doc/world-simulator/prd.project.md`

## 主题目录
- `viewer/`: Viewer 与 Web/交互/可视化相关设计。
- `llm/`: LLM 行为、Prompt、评估与稳定性相关设计。
- `launcher/`: 启动器与链路编排相关设计。
- `scenario/`: 场景定义、初始化与配置相关设计。
- `kernel/`: 内核规则桥接与 WASM 规则执行相关设计。
- `archive/`: 历史归档文档。
- `m4/`: M4 专题文档。

## 专项手册
- Viewer 使用手册：`doc/world-simulator/viewer/viewer-manual.md`

## 维护约定
- 新文档按主题目录落位，不再默认平铺在模块根目录。
- 模块行为变更需同步更新 `prd.md` 与 `prd.project.md`。
