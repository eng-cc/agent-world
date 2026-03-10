# 启动器反馈入口设计（2026-03-02）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-feedback-entry-2026-03-02.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-feedback-entry-2026-03-02.project.md`

## 1. 设计定位
定义桌面启动器内的最小反馈采集闭环：在本地 UI 中收集 `Bug / Suggestion`，并将反馈内容连同配置快照与最近日志落成可追踪 JSON 包。

## 2. 设计结构
- 交互输入层：提供反馈类型、标题、描述与输出目录输入。
- 落盘封装层：序列化为稳定 JSON，文件名带时间戳与类别。
- 上下文附带层：自动携带 `LaunchConfig` 快照与最近固定行数日志。
- 失败诊断层：目录权限或写入失败时返回明确错误，允许用户修改目录后重试。

## 3. 关键接口 / 入口
- `feedback_entry.rs`
- `kind/title/description/output_dir`
- `{YYYYMMDDTHHMMSSZ}-{kind}.json`
- `created_at` / `launcher_config` / `recent_logs`

## 4. 约束与边界
- 首版只落本地反馈包，不接入远端反馈 API。
- 日志附带必须有上限，避免反馈文件无限膨胀。
- 文件命名与时间戳格式要稳定、可测试。
- 主文件行数必须受控，反馈逻辑优先拆到独立模块。

## 5. 设计演进计划
- 先补 UI 反馈入口与 JSON 结构。
- 再补日志/配置附带与失败提示。
- 最后通过单测与文档回写收口桌面端反馈闭环。
