# Viewer 贴图查看器（可截图）设计文档

## 目标
- 提供一个“专门看贴图效果”的查看器入口，避免在完整世界场景里手动找角度。
- 支持一键截图留证据，便于美术/技术美术做贴图回归对比。
- 复用现有 `capture-viewer-frame.sh` 闭环，不引入新的图形运行时依赖。

## 范围

### In Scope
- 新增脚本：`scripts/viewer-texture-inspector.sh`。
- 支持按“贴图来源实体”检查：`agent/location/asset/power_plant/power_storage`。
- 支持材质变体批量检查：`default/matte/glossy`。
- 支持输出截图、日志、元数据到独立目录（可归档对比）。
- 支持覆盖 base/normal/metallic_roughness/emissive 贴图路径，便于临时验图。

### Out of Scope
- 不新增 Viewer 运行中 UI 面板（本期先做脚本化闭环）。
- 不改 world 模拟协议与 `viewer` 网络协议。
- 不做贴图热重载 watcher（后续阶段再补）。

## 接口 / 数据
- 新脚本参数（草案）：
  - `--preset-file <path>`：主题预设 env 文件（默认 industrial_default）。
  - `--inspect <entity|all>`：贴图来源实体（默认 `all`）。
  - `--variants <list>`：`default,matte,glossy,all`。
  - `--scenario <name>`：截图场景（默认 `llm_bootstrap`）。
  - `--out-dir <dir>`：输出目录。
  - `--base-texture/--normal-texture/--mr-texture/--emissive-texture`：临时覆盖贴图。
- 脚本内部策略：
  - 使用统一“观察载体”（location 槽位 + 固定镜头）渲染，保证不同来源贴图可在一致视角下对比。
  - 通过环境变量把待检查贴图注入 `AGENT_WORLD_VIEWER_LOCATION_*_TEXTURE_ASSET`。
- 产物目录（计划）：
  - `output/texture_inspector/<timestamp>/<entity>/<variant>/`
  - 每项包含：`viewer.png`、`viewer.log`、`live_server.log`、`meta.txt`。

## 里程碑
- VTI-0：设计文档 + 项目管理文档建档。
- VTI-1：实现 `viewer-texture-inspector.sh`（参数解析、贴图映射、截图导出）。
- VTI-2：测试验证、手册更新、devlog 回写、状态收口。

## 风险
- 场景对象在画面中构图不稳定：
  - 缓解：固定 `first_location` 自动聚焦 + 固定 automation steps。
- 不同实体贴图应用到统一载体后与真实实体形态存在偏差：
  - 缓解：文档明确“该工具用于贴图风格/对比验证，不替代最终场景验收”。
- 多次批量截图耗时：
  - 缓解：支持 `--inspect`/`--variants` 精简运行范围。
