# Agent World: 启动链路脚本迁移（2026-02-28）设计

- 对应需求文档: `doc/testing/launcher/launcher-chain-script-migration-2026-02-28.prd.md`
- 对应项目管理文档: `doc/testing/launcher/launcher-chain-script-migration-2026-02-28.project.md`

## 1. 设计定位
定义启动器测试链路专题设计，统一脚本迁移、生命周期校验、配置继承与可用性验收。

## 2. 设计结构
- 测试场景层：围绕 launcher 主路径定义链路、配置与交互场景。
- 自动化执行层：通过脚本或闭环用例验证启动器行为。
- 配置继承层：对 viewer/auth/node 配置自动连线与生命周期状态做校验。
- 验收回写层：沉淀可用性审查结果与失败签名。

## 3. 关键接口 / 入口
- launcher 测试脚本/入口
- 配置继承与自动连线点
- 生命周期状态与 readiness
- launcher 验收用例

## 4. 约束与边界
- 测试设计必须覆盖真实启动器主路径。
- 自动继承不得掩盖配置错误。
- 不在本专题扩展新的 launcher 产品能力。

## 5. 设计演进计划
- 先固化 launcher 场景。
- 再补自动化执行与配置校验。
- 最后完成可用性验收与回写。
