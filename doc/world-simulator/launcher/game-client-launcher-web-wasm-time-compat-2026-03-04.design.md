# 启动器 Web Wasm 时间兼容设计

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.project.md`

## 1. 设计定位
定义 launcher 在 wasm/web 目标下的时间来源、兼容策略与异常回退机制。

## 2. 设计结构
- 时间抽象层：封装 web 环境可用的时间来源。
- 兼容层：替换不可用 `SystemTime`/阻塞式时间调用。
- 诊断层：为时间相关异常提供可观测签名与回退路径。

## 3. 关键接口 / 入口
- launcher 时间工具/适配层
- web 启动器初始化与轮询逻辑
- 错误签名与回归脚本入口

## 4. 约束与边界
- web 端不得再触发 `time not implemented` 类错误。
- 时间兼容修复不能破坏 native 端逻辑。
- 相关异常必须能通过回归脚本稳定复现/验证。

## 5. 设计演进计划
- 先完成设计补齐与互链回写。
- 再按项目文档任务拆解推进实现与回归。
