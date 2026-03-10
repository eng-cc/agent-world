# README WASM 主链路收口：Live 模块执行 + 默认持久化模块仓库 + 模块实例化 + 升级动作（设计文档）设计

- 对应需求文档: `doc/readme/gap/readme-gap-wasm-live-persistence-instance-upgrade.prd.md`
- 对应项目管理文档: `doc/readme/gap/readme-gap-wasm-live-persistence-instance-upgrade.project.md`

## 1. 设计定位
定义 README 中 WASM 主链路收口设计，统一 live 模块执行、默认持久化仓库、实例化与升级动作口径。

## 2. 设计结构
- WASM 主链路层：定义 live 执行、实例化与升级动作的主路径。
- 持久化仓库层：明确默认模块仓库与状态持久化职责。
- 升级动作层：统一实例升级、迁移与兼容说明。
- 口径收口层：把 README 描述与正式专题对齐。

## 3. 关键接口 / 入口
- WASM live 执行入口
- 默认持久化仓库
- 模块实例化/升级动作
- README 收口清单

## 4. 约束与边界
- README 需聚焦稳定外部口径。
- 升级动作口径必须兼容已发布模块。
- 不在本专题细写全部实现步骤。

## 5. 设计演进计划
- 先收敛主链路描述。
- 再补持久化与升级边界。
- 最后统一 README 与专题引用。
