# oasis7 Runtime：WASM 模块设计对齐增强（agent-os 借鉴）设计

- 对应需求文档: `doc/world-runtime/wasm/wasm-agent-os-alignment-hardening.prd.md`
- 对应项目管理文档: `doc/world-runtime/wasm/wasm-agent-os-alignment-hardening.project.md`

## 1. 设计定位
定义 WASM 模块设计对齐增强方案，借鉴 agent-os 经验收敛模块边界、宿主接口与运行治理。

## 2. 设计结构
- 能力对齐层：对齐 agent-os 风格的模块边界、宿主能力与调用方式。
- 接口收敛层：梳理 host ABI、模块 manifest 与能力声明。
- 治理加固层：把发布、启用与权限约束纳入统一治理。
- 迁移验证层：为既有模块提供对齐迁移与回归路径。

## 3. 关键接口 / 入口
- WASM host ABI
- 模块 manifest/能力声明
- 治理启用入口
- 对齐迁移回归

## 4. 约束与边界
- 对齐增强不能破坏现有主路径兼容性。
- 能力声明必须显式、最小化。
- 不在本专题整体替换 WASM 运行时。

## 5. 设计演进计划
- 先梳理 agent-os 对齐目标。
- 再收敛接口与治理约束。
- 最后执行迁移与回归。
