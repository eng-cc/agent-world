# 启动器 Native 遗留代码清理设计（2026-03-06）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-native-legacy-cleanup-2026-03-06.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-native-legacy-cleanup-2026-03-06.project.md`

## 1. 设计定位
定义 native 启动器在迁移完成后的遗留状态字段、无效常量边界与未被入口引用测试资产的清理方案，在不改变运行时行为的前提下降低维护噪声。

## 2. 设计结构
- 状态字段清理层：移除无读写路径的历史状态与初始化逻辑。
- 测试资产清理层：删除未被编译入口引用的旧测试文件。
- 常量收敛层：整理平台边界与历史常量，消除明显残留告警。
- 回归保护层：使用现有 required 测试确认 native/web 行为不漂移。

## 3. 关键接口 / 入口
- `oasis7_client_launcher` native 状态结构
- 未引用测试文件与旧常量定义
- `test_tier_required` 回归链路

## 4. 约束与边界
- 清理只针对无读写路径或未引用资产，不能误删仍在使用的兼容入口。
- 不改变 native 启动器对外功能语义。
- 清理后仍需保持 web/native 共用逻辑可编译。
- 本阶段不引入新的架构重组，只做噪声收口。

## 5. 设计演进计划
- 先识别并冻结遗留噪声清单。
- 再逐项移除状态字段、旧测试和历史常量。
- 最后用 required 回归与文档回写收口。
