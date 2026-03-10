# README 生产级收口：LLM 制度动作 + DistFS 状态主路径 + 去中心化默认拓扑（设计文档）设计

- 对应需求文档: `doc/readme/production/readme-prod-closure-llm-distfs-consensus.prd.md`
- 对应项目管理文档: `doc/readme/production/readme-prod-closure-llm-distfs-consensus.project.md`

## 1. 设计定位
定义 README 生产级收口设计，统一 LLM 制度动作、DistFS 状态主路径与去中心化默认拓扑口径。

## 2. 设计结构
- 制度动作层：说明 LLM 在生产制度动作中的边界。
- DistFS 主路径层：统一状态同步与存储主路径说明。
- 默认拓扑层：定义去中心化默认拓扑及其适用条件。
- README 对齐层：把三条生产口径收敛到统一入口。

## 3. 关键接口 / 入口
- LLM 制度动作说明
- DistFS 状态主路径
- 默认拓扑口径
- README 收口检查项

## 4. 约束与边界
- 三条生产口径必须互不冲突。
- README 只保留稳定对外表达。
- 不在本专题细化底层协议。

## 5. 设计演进计划
- 先冻结三条主口径。
- 再统一 README 描述。
- 最后验证专题互链。
