# Compound/Hardware 内建资源硬迁移设计

- 对应需求文档: `doc/world-simulator/kernel/resource-kind-compound-hardware-hard-migration.prd.md`
- 对应项目管理文档: `doc/world-simulator/kernel/resource-kind-compound-hardware-hard-migration.project.md`

## 1. 设计定位
定义内建 `ResourceKind` 从四元模型收敛到 `Electricity + Data` 的硬迁移方案，将 `Compound/Hardware` 彻底从核心层移除，并将相关语义转交 WASM 模块层维护。

## 2. 设计结构
- 资源模型层：`ResourceKind` 枚举与解析器移除 `Compound`、`Hardware` 两个内建项。
- 核心实现层：simulator/runtime/viewer 仅处理最小内建资源集，删除旧分支与展示逻辑。
- 提示与文档层：LLM 提示、README 与示例不再声明 `compound/hardware` 为内建资源。
- 测试迁移层：修订依赖旧资源模型的断言，确保 required 门禁保持可追溯。

## 3. 关键接口 / 入口
- `ResourceKind`
- 资源 kind 解析与匹配逻辑
- viewer 资源展示统计代码
- `crates/agent_world/src/simulator/llm_agent/*`
- `README.md` / 模块文档资源模型说明

## 4. 约束与边界
- 采用无兼容硬迁移策略，不保留旧语义别名或自动升级路径。
- 核心层必须拒绝 `compound/hardware` 作为内建资源输入。
- 本阶段不直接实现完整 WASM 资产标准接口，只完成内核侧退场。
- 历史文档若提及旧模型，以当前活跃 README 与实现语义为准逐步收敛。

## 5. 设计演进计划
- 先冻结“最小内建资源集”边界。
- 再完成枚举、解析、viewer 与提示层清理。
- 最后以测试与文档回写收口，确保仓库主口径与实现一致。
