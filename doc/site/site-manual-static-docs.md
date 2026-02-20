# Site 使用手册静态化（CN/EN）设计文档

## 目标
- 在 `site/doc/cn` 与 `site/doc/en` 下建立可直接发布到 GitHub Pages 的手册站框架。
- 将现有 Viewer 用户手册整理为站内可访问内容，形成“首页 -> 文档目录 -> 手册正文”的闭环。
- 保持当前纯静态部署模式（无需新增构建系统）。

## 范围
- 范围内
  - 新增文档目录页（CN/EN）与基础布局框架。
  - 新增手册正文页（CN/EN）并接入站点语言切换。
  - 在现有中英文首页增加“使用手册”入口。
  - 补充文档页样式与最小交互脚本。
- 范围外
  - 引入 SSG（VitePress/MkDocs）或服务端渲染。
  - 一次性迁移全部 `doc/` 技术文档。
  - 改动主站现有信息架构与交互主流程。

## 接口/数据
- 页面目录
  - `site/doc/cn/index.html`
  - `site/doc/en/index.html`
  - `site/doc/cn/viewer-manual.html`
  - `site/doc/en/viewer-manual.html`
- 样式/交互
  - `site/assets/styles.css`
  - `site/assets/app.js`
- 入口页面
  - `site/index.html`
  - `site/en/index.html`
- 内容来源
  - `doc/viewer-manual.md`（中文原稿）

## 里程碑
- M1：文档与任务拆解
  - 新增本设计文档与项目管理文档。
- M2：文档框架上线
  - 完成 `site/doc/cn|en` 目录页与导航基础能力。
- M3：手册内容整理
  - 完成 CN/EN 手册正文接入与链接互通。
- M4：验证收口
  - 完成静态链接自检、`cargo check`、项目文档回写与 devlog。

## 风险
- 风险：CN/EN 内容漂移。
  - 缓解：同任务内成对维护 `cn/en` 页面，并在目录页统一挂载。
- 风险：手册内容更新后站内版本过时。
  - 缓解：明确 `doc/viewer-manual.md` 为内容基线，后续按任务滚动同步。
- 风险：文档页样式影响首页样式。
  - 缓解：新增样式尽量限定在 `.docs-*` 命名空间内。
