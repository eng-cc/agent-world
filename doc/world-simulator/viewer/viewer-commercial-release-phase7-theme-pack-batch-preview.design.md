# Viewer 商业化发行 Phase 7 主题包批量预览设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase7-theme-pack-batch-preview.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-commercial-release-phase7-theme-pack-batch-preview.project.md`

## 1. 设计定位
定义首批工业风主题包与批量预览流程：把“手工切环境变量调参”升级为“可复跑、可留证据”的主题预览与截图输出链路。

## 2. 设计结构
- 资产包层：在 `assets/themes/industrial_v1/` 下组织 mesh、PBR 贴图和预设 env。
- 预设入口层：统一管理外部 mesh/贴图/材质变体环境变量。
- 批量预览层：脚本一次执行多变体截图并输出到带时间戳目录。
- 回归层：通过默认镜头构图修复和输出留痕支撑版本对比。

## 3. 关键接口 / 入口
- `assets/themes/industrial_v1/`
- `presets/*.env`
- `scripts/viewer-theme-pack-preview.sh`
- `output/theme_preview/<timestamp>/<variant>/`
- `capture-viewer-frame.sh`

## 4. 约束与边界
- 首版统一使用兼容性更强的 `.gltf + .bin + .png` 组合。
- 默认不启用主题包，保持当前 Viewer 表现不变。
- 本阶段不引入运行中热重载 watcher。
- 主题切换导致的视觉退化需通过预览输出可审计地发现。

## 5. 设计演进计划
- 先落地工业风主题包资产与预设。
- 再实现多变体批量预览与截图输出。
- 最后以测试、手册和镜头构图修复收口 Phase 7。
