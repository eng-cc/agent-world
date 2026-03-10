# README 收口：基础设施模块执行引擎 + 编译 Sandbox 隔离（设计文档）设计

- 对应需求文档: `doc/readme/gap/readme-gap-infra-exec-compiler-sandbox.prd.md`
- 对应项目管理文档: `doc/readme/gap/readme-gap-infra-exec-compiler-sandbox.project.md`

## 1. 设计定位
定义基础设施模块执行引擎与编译 Sandbox 隔离的 README 收口设计，统一执行、编译与隔离边界。

## 2. 设计结构
- 执行引擎层：明确基础设施模块的执行入口与责任。
- 编译隔离层：定义编译 Sandbox 的边界、输入输出与限制。
- 协同关系层：说明执行与编译链路如何衔接。
- 口径验证层：确保 README 与专题文档一致。

## 3. 关键接口 / 入口
- 执行引擎入口
- 编译 Sandbox 边界
- 执行/编译衔接点
- README 口径检查项

## 4. 约束与边界
- README 只描述稳定对外口径。
- 隔离边界必须清晰且可审计。
- 不在本专题扩展新的编译后端。

## 5. 设计演进计划
- 先固化执行/编译职责。
- 再统一 Sandbox 隔离描述。
- 最后完成 README 回写。
