# Agent World Simulator：世界初始化（项目管理文档）

## 任务拆解
- [x] I1 定义初始化配置结构（WorldInitConfig/Origin/Dust/Agent）
- [x] I1 实现世界初始化输出（WorldModel + Report）
- [x] I2 提供 WorldKernel 便捷构造接口并接入校验
- [x] I2 补充初始化单元测试（默认流程/确定性/错误分支）
- [x] 文档更新：同步设计分册与导出入口
- [x] I3 支持自定义地点列表（LocationSeedConfig）
- [x] I3 支持初始资源配置（Origin/Location/Agent）
- [x] I3 补充资源/多地点初始化测试
- [x] I4 支持电力设施种子（PowerPlant/PowerStorage）
- [x] I4 增加设施参数校验与错误分支
- [x] I4 补充电力设施初始化测试
- [x] I5 提供场景模板（WorldScenario）
- [x] I5 提供示例工具（world_init_demo）
- [x] I6 扩展场景模板（resource_bootstrap 初始库存）
- [x] I7 README 补充示例工具说明（world_init_demo）
- [x] I8 扩展场景模板（twin_region_bootstrap 多区域）
- [x] I9 补充文档使用示例与 demo 帮助输出
- [x] I10 demo 输出地点资源摘要
- [x] I11 扩展场景模板（triad_region_bootstrap 多区域）
- [x] I12 demo 输出 Agent 资源摘要
- [x] I13 扩展场景模板（dusty_bootstrap 启用尘埃云）
- [x] I14 补充场景别名解析测试
- [x] I15 demo 输出尘埃碎片数量
- [x] I16 demo 输出地点设施统计
- [x] I17 文档补充场景使用建议
- [x] I18 文档补充场景别名说明

## 依赖
- `generate_fragments`（尘埃云生成器）
- `WorldKernel` / `WorldModel` 基础结构

## 状态
- 当前阶段：I18（别名说明完成）
