# Viewer UI 多语言与字体回退设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-i18n.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-i18n.project.md`

## 1. 设计定位
定义 Viewer UI 的中英双语、词典回退和中文字体渲染方案：统一所有 UI 文案入口，并保证 `zh-CN` / `en-US` 切换可测、可回退、可读。

## 2. 设计结构
- 语言模型层：`UiLocale` 与 `UiI18n` 负责当前语言、词条查询和模板格式化。
- 词典治理层：`catalog_zh_cn` / `catalog_en_us` 保持同构 key 集，并遵循当前语言 -> `en-US` -> key 的回退链。
- 交互入口层：Top Controls 提供显式语言切换，默认 `zh-CN`，可选持久化。
- 字体兼容层：引入 `fonts/ms-yahei.ttf` 解决 UI/3D 标签中文 tofu 问题。

## 3. 关键接口 / 入口
- `UiLocale`
- `UiI18n`
- `main.rs` / `ui_text.rs` / `diagnosis.rs` / `timeline_controls.rs`
- `fonts/ms-yahei.ttf`

## 4. 约束与边界
- 首版只做中英双语，不跟随系统语言，也不引入第三方 i18n 框架。
- UI 中禁止直接写最终显示文案，必须走 key/模板查询。
- 中英文模板参数集合必须一致，避免格式化时单侧缺参。
- 字体修复只解决渲染可读性，不改变“界面内手动切换语言”的策略。

## 5. 设计演进计划
- 先盘点 key 并建立 `UiLocale` / `UiI18n` 基础设施。
- 再迁移各模块文案与语言切换入口。
- 最后补字体资源、回退测试和截图闭环。
