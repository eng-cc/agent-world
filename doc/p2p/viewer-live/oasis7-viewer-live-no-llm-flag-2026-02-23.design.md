# oasis7_viewer_live `--no-llm` 关闭开关设计文档（2026-02-23）设计

- 对应需求文档: `doc/p2p/viewer-live/oasis7-viewer-live-no-llm-flag-2026-02-23.prd.md`
- 对应项目管理文档: `doc/p2p/viewer-live/oasis7-viewer-live-no-llm-flag-2026-02-23.project.md`

## 1. 设计定位
定义 `oasis7_viewer_live --no-llm` 关闭开关设计，确保在发行基线下仍能显式关闭 LLM 路径并保持行为可预测。

## 2. 设计结构
- CLI 开关层：定义 `--no-llm` 的参数语义与优先级。
- 启动判定层：在默认开启前提下处理显式关闭分支。
- 运行降级层：关闭 LLM 后切到无模型体验与相应提示。
- 验证回写层：沉淀 flag 行为回归与发行说明。

## 3. 关键接口 / 入口
- `--no-llm` CLI 参数
- 启动参数判定入口
- 无 LLM 运行路径
- flag 回归用例

## 4. 约束与边界
- 显式 flag 优先级必须高于默认值。
- 关闭后不得残留隐式 LLM 依赖。
- 不在本专题扩展更多运行模式。

## 5. 设计演进计划
- 先固化 CLI 语义。
- 再接启动判定与降级。
- 最后补回归和说明文档。
