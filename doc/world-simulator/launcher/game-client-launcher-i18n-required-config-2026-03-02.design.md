# 启动器中英文切换与必填配置校验设计（2026-03-02）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.project.md`

## 1. 设计定位
定义桌面启动器的双语界面状态与启动前阻断校验机制，使不同语言用户都能在真正启动前识别并修复配置缺失或格式错误。

## 2. 设计结构
- 语言状态层：以 `UiLanguage` 统一管理 `ZhCn/EnUs` 文案选择。
- 文案渲染层：标题、字段、按钮、状态与关键提示都通过语言状态即时渲染。
- 配置校验层：`collect_required_config_issues(&LaunchConfig)` 收集阻断问题列表。
- 门控反馈层：存在阻断项时显示双语错误清单并禁用启动按钮。

## 3. 关键接口 / 入口
- `UiLanguage` (`ZhCn` / `EnUs`)
- `AGENT_WORLD_CLIENT_LAUNCHER_LANG`
- `collect_required_config_issues(&LaunchConfig)`
- 启动前错误清单与按钮禁用态

## 4. 约束与边界
- 语言切换必须即时生效，不依赖外部资源包。
- 校验规则需尽量复用现有解析逻辑，避免前后端分叉。
- UI 阻断只是前置保护，内部 preflight 仍需保留二次校验。
- 本阶段不扩展复杂配置向导流程。

## 5. 设计演进计划
- 先固定语言枚举与默认判定策略。
- 再补双语文案渲染和配置阻断清单。
- 最后用单测与 required 回归确认语言与校验语义稳定。
