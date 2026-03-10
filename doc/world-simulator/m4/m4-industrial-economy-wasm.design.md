# M4 社会经济系统：工业链路与 WASM 模块化（Recipe/Product/Factory）设计

- 对应需求文档: `doc/world-simulator/m4/m4-industrial-economy-wasm.prd.md`
- 对应项目管理文档: `doc/world-simulator/m4/m4-industrial-economy-wasm.project.md`

## 1. 设计定位
定义 M4 工业链路与 WASM 模块化设计，把 Recipe/Product/Factory 结构收敛到可演进模块体系。

## 2. 设计结构
- 经济建模层：定义产品、配方、工厂与资源关系。
- WASM 模块层：把工业规则、生产行为和状态封装到模块。
- 运行接线层：让工业模块进入世界运行时主路径。
- 可玩性验证层：围绕工业闭环执行体验与稳定性验证。

## 3. 关键接口 / 入口
- Recipe/Product/Factory 模型
- 工业 wasm 模块入口
- 运行时接线点
- 工业闭环验证场景

## 4. 约束与边界
- 模块化不能破坏基础工业语义。
- 产品链路需保持可追踪。
- 不在本专题覆盖全部市场与治理细节。

## 5. 设计演进计划
- 先固化经济模型。
- 再接 wasm 模块化。
- 最后联动可玩性验证。
