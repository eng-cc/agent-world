# 规则 Wasm 模块装载治理设计

- 对应需求文档: `doc/world-simulator/kernel/kernel-rule-wasm-module-governance.prd.md`
- 对应项目管理文档: `doc/world-simulator/kernel/kernel-rule-wasm-module-governance.project.md`

## 1. 设计定位
定义规则 wasm artifact 注册表、按 hash 激活 API 与装载治理边界。

## 2. 设计结构
- 注册表层：维护 wasm hash 到 bytes 的受控映射。
- 装载层：通过按 hash 激活 API 将规则模块绑定到内核 evaluator。
- 测试层：覆盖缺失 hash、冲突注册与成功激活路径。

## 3. 关键接口 / 入口
- `set_*_from_registry(...)` / `remove_pre_action_wasm_rule_artifact(...)`
- wasm artifact 注册表与查找接口
- 内核规则 evaluator 安装链路

## 4. 约束与边界
- 缺失 hash 时必须显式报错。
- 同 hash 不允许映射不同 bytes。
- 本阶段注册表仅内存态，重启恢复后续处理。

## 5. 设计演进计划
- 先保持现有规则语义稳定。
- 再沿项目文档任务拆解推进实现与回归。
