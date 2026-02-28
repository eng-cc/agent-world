# Viewer Texture Inspector Art Capture 优化（2026-02-28）

## 目标
- 让 `viewer-texture-inspector` 产出的截图可直接用于美术评审，而不是仅用于“链路联通确认”。
- 在不同资源类型（`agent/location/asset/power_plant/power_storage`）之间提供更稳定的主体构图。
- 降低 UI 面板与任务层遮挡对材质对比（`default/matte/glossy`）的干扰。
- 对关键实体（`power_plant/power_storage`）建立变体可区分性门禁，避免产出“同图不同名”结果。
- 引入“视觉近似失败”判定（SSIM 阈值），把“非同图”与“可判读”区分开。

## 范围
- **范围内**
  - 改造 `scripts/viewer-texture-inspector.sh`，新增“美术截图模式（art capture）”。
  - 为不同资源类型配置独立镜头自动化步骤（`focus/orbit/zoom/pan`）。
  - 在保留原始窗口截图 `viewer.png` 的同时，新增裁切图 `viewer_art.png`。
  - 新增 closeup 二次抓图产物（`viewer_closeup.png`、`viewer_art_closeup.png`）。
  - 为 `power_plant/power_storage` 增加三变体一致性校验与重试策略。
  - 为 art-capture 新增材质评审灯光口径（低 bloom + 稳定曝光 + 强化轮廓光）。
  - 为 power 实体改用实体目标焦点（`first_power_plant` / `first_power_storage`）。
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
  - `--closeup-automation-steps <steps>`：覆盖 closeup 自动化步骤。
  - `--art-lighting` / `--no-art-lighting`：控制评审灯光口径。
  - `--variant-ssim-threshold <f>`：控制变体视觉近似门禁阈值（0..1）。
  - `--crop-window <w:h:x:y>`：覆盖裁切窗口。
- 输出目录：
  - `output/texture_inspector/<timestamp>/<entity>/<variant>/viewer.png`
  - `output/texture_inspector/<timestamp>/<entity>/<variant>/viewer_art.png`（新增）
  - `output/texture_inspector/<timestamp>/<entity>/<variant>/viewer_closeup.png`（新增）
  - `output/texture_inspector/<timestamp>/<entity>/<variant>/viewer_art_closeup.png`（新增）
  - `output/texture_inspector/<timestamp>/<entity>/variant_validation.txt`（新增）
  - `meta.txt`（新增记录镜头与裁切配置）

## 里程碑
- **T0 建档**：设计文档 + 项目管理文档。
- **T1 实现**：脚本参数、分实体镜头、裁切产物与元数据落地。
- **T2 验证与收口**：脚本语法/help/实跑验证，更新项目文档与开发日志。
- **T3 变体校验增强**：closeup 双图与 power 实体一致性门禁/重试落地。
- **T4 视觉调参**：修复镜头贴脸问题并完成全量复跑与人工复核。
- **T5 视觉门禁强化**：实体焦点 + 灯光口径 + SSIM 阈值门禁落地与全量验证。

## 风险
- **镜头过近或过远**：不同 mesh 尺度差异可能导致裁切后主体不完整。
  - 缓解：提供实体级默认镜头，并允许 `--automation-steps` 覆盖。
- **镜头倍率误配导致“平面贴脸”**：会让三变体差异不可读，即使哈希不一致也缺乏评审价值。
  - 缓解：将 art-capture 预设 zoom 调整为可读近中景（`>1`），并保留 closeup fallback 角度。
- **“哈希不同但视觉仍相近”漏检风险**：会误判为可评审样本。
  - 缓解：新增 `min_pair_ssim` 与阈值门禁，触发重试并可失败出具。
- **裁切区域误伤主体**：固定裁切窗口在极端场景可能截掉目标。
  - 缓解：提供 `--crop-window` 覆盖并记录到 `meta.txt`。
- **运行环境依赖差异**：裁切依赖 `ffmpeg` 过滤表达式，环境差异可能导致失败。
  - 缓解：失败时保留 `viewer.png` 并显式告警，不中断整批产出。
