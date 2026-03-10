# README 缺口 2 收口：LLM 直连 WASM 生命周期（设计文档）设计

- 对应需求文档: `doc/readme/gap/readme-gap2-llm-wasm-lifecycle.prd.md`
- 对应项目管理文档: `doc/readme/gap/readme-gap2-llm-wasm-lifecycle.project.md`

## 1. 设计定位
定义 README 缺口 2 收口设计，统一 LLM 直连 WASM 生命周期的对外表达。

## 2. 设计结构
- 生命周期阶段层：定义 LLM 直连 WASM 的创建、运行、更新与结束阶段。
- 触发关系层：解释 LLM 行为与 WASM 生命周期的衔接。
- 治理边界层：说明权限、发布和回滚约束。
- README 收口层：同步专题入口与外部口径。

## 3. 关键接口 / 入口
- LLM-WASM 生命周期阶段
- 触发/衔接入口
- 治理边界说明
- README 校验项

## 4. 约束与边界
- 生命周期表达需与正式专题一致。
- README 不承载实现型调度细节。
- 不在本专题扩展新的编排机制。

## 5. 设计演进计划
- 先固定生命周期阶段。
- 再对齐触发与治理边界。
- 最后完成 README 回写。
