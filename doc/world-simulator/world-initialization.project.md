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

## 依赖
- `generate_fragments`（尘埃云生成器）
- `WorldKernel` / `WorldModel` 基础结构

## 状态
- 当前阶段：I3（资源/多地点初始化完成）
