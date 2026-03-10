# 启动器设置中心与 LLM 配置面板设计

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-llm-settings-panel-2026-03-02.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-llm-settings-panel-2026-03-02.project.md`

## 1. 设计定位
定义启动器设置中心的字段分层、TOML 映射、保存/重载语义与错误反馈模型。

## 2. 设计结构
- 设置分层：游戏、区块链、LLM 三类配置统一收口。
- 文件映射：`config.toml` 中 `[llm]` 及相关配置的读写边界。
- 交互层：即时生效字段与显式保存字段的区分。

## 3. 关键接口 / 入口
- 设置入口按钮与 settings window
- `config.toml` 读写逻辑与字段映射
- 保存/重载/清空的 UI 反馈接口

## 4. 约束与边界
- 仅更新约定字段，避免破坏无关 TOML 结构。
- 空字符串保存需明确表示删除配置键。
- 非法 TOML 必须在 UI 中给出可操作错误反馈。

## 5. 设计演进计划
- 先完成设计补齐与互链回写。
- 再按项目文档任务拆解推进实现与回归。
