# Viewer Live 旧 Tick 驱动文档归档设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-tick-driven-doc-archive-2026-02-27.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-tick-driven-doc-archive-2026-02-27.project.md`

## 1. 设计定位
定义旧 tick-driven 阶段文档的归档与替代链收口方案：移除已失效的 Phase 1~7 活跃入口，只保留 event-driven 阶段作为当前有效文档。

## 2. 设计结构
- 归档对象层：清理 Viewer Live Phase 1~7 的设计与项目管理文档活跃入口。
- 替代链层：在索引中明确由 event-drive 相关阶段文档承接当前权威语义。
- 边界说明层：Phase 8~10 保留为当前有效 event-driven 记录，历史 devlog 不重写。
- 治理收口层：同步更新索引、说明文档和项目状态，避免误读。

## 3. 关键接口 / 入口
- Viewer Live Phase 1~7 历史专题
- event-driven 阶段文档入口
- `prd.index.md` / 模块 README / 项目台账

## 4. 约束与边界
- 只处理文档活跃入口和替代链，不改历史 devlog 内容。
- Phase 8~10 作为当前有效收口记录必须保留。
- 归档后需要留下明确替代入口，不能只做删除不做指向。
- 本专题是文档治理，不承担代码链路改造。

## 5. 设计演进计划
- 先识别旧 tick-driven 活跃入口。
- 再补 event-driven 替代链和索引说明。
- 最后清理活跃文档树中的历史误导入口。
