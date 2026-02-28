# Viewer Texture Inspector Art Capture 优化（2026-02-28）

## 目标
- 让 `viewer-texture-inspector` 产出的截图可直接用于美术评审，而不是仅用于“链路联通确认”。
- 在不同资源类型（`agent/location/asset/power_plant/power_storage`）之间提供更稳定的主体构图。
- 降低 UI 面板与任务层遮挡对材质对比（`default/matte/glossy`）的干扰。

## 范围
- **范围内**
  - 改造 `scripts/viewer-texture-inspector.sh`，新增“美术截图模式（art capture）”。
  - 为不同资源类型配置独立镜头自动化步骤（`focus/orbit/zoom/pan`）。
  - 在保留原始窗口截图 `viewer.png` 的同时，新增裁切图 `viewer_art.png`。
  - 将镜头参数与裁切参数写入每组 `meta.txt`，便于复盘。
- **范围外**
  - 不改动 `world_viewer_live` 协议与运行时数据结构。
  - 不改动 viewer 渲染核心（材质/光照/后处理）实现。
  - 不引入新的 E2E 测试框架，仅补脚本级验证闭环。

## 接口 / 数据
- 脚本：`scripts/viewer-texture-inspector.sh`
- 新参数（计划）：
  - `--art-capture`：启用美术评审友好的镜头与裁切产物。
  - `--automation-steps <steps>`：覆盖默认自动化步骤。
  - `--crop-window <w:h:x:y>`：覆盖裁切窗口。
- 输出目录：
  - `output/texture_inspector/<timestamp>/<entity>/<variant>/viewer.png`
  - `output/texture_inspector/<timestamp>/<entity>/<variant>/viewer_art.png`（新增）
  - `meta.txt`（新增记录镜头与裁切配置）

## 里程碑
- **T0 建档**：设计文档 + 项目管理文档。
- **T1 实现**：脚本参数、分实体镜头、裁切产物与元数据落地。
- **T2 验证与收口**：脚本语法/help/实跑验证，更新项目文档与开发日志。

## 风险
- **镜头过近或过远**：不同 mesh 尺度差异可能导致裁切后主体不完整。
  - 缓解：提供实体级默认镜头，并允许 `--automation-steps` 覆盖。
- **裁切区域误伤主体**：固定裁切窗口在极端场景可能截掉目标。
  - 缓解：提供 `--crop-window` 覆盖并记录到 `meta.txt`。
- **运行环境依赖差异**：裁切依赖 `ffmpeg` 过滤表达式，环境差异可能导致失败。
  - 缓解：失败时保留 `viewer.png` 并显式告警，不中断整批产出。
