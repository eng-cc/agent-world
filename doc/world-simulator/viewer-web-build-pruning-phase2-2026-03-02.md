# Viewer Web 构建体积裁剪 Phase 2（2026-03-02）

## 目标
- 在 Phase 1 模块裁剪基础上，进一步降低 web 产物体积，重点压缩 wasm 主体大小。
- 优先通过“功能特性精细化 + 嵌入资源去内联”减少不必要编译与链接负担。
- 保持 viewer 在 native / wasm 两端构建通过，且核心 UI/3D 功能不回退。

## 范围
- `crates/agent_world_viewer/Cargo.toml`
  - 将 `bevy` 从默认特性切换为显式最小特性集，移除不需要的默认模块（如 audio/gamepad）。
- `crates/agent_world_viewer/src/copyable_text.rs`
  - 移除 `include_bytes!` 方式内联大字体，改为运行时资源加载（AssetServer），避免将字体字节打进 wasm。
- `crates/agent_world_viewer/src/main_ui_runtime.rs`、`crates/agent_world_viewer/src/main.rs`
  - 同步字体加载链路与调用点，确保 wasm/native 路径一致可用。

## 接口 / 数据
- 体积口径：
  - `trunk build --release` 后比较 `*_bg.wasm` 大小。
  - 对比 dist 总大小（wasm + js + assets）用于评估“转移体积”风险。
- 构建回归：
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown`
- 风险口径：
  - 若字体改为外部资源，可能导致首次加载时字体可用性与兜底字体差异。

## 里程碑
- M0：完成建档（本文件 + `.project.md`）。
- M1：完成 `bevy` feature 精细化与编译回归。
- M2：完成字体从内联切换到资产加载并回归。
- M3：完成 trunk release 体积对比与文档收口。

## 风险
- `bevy` 特性裁剪过度导致运行时组件缺失：
  - 用 native/wasm 双 check 兜底，并保留必要渲染链路特性。
- 字体改外部资源导致中文显示退化或加载竞态：
  - 保留默认字体兜底，中文字体加载失败时不阻塞主流程。
- wasm 缩小但 dist 总体积不降：
  - 同时统计 wasm 与 dist，明确“缩 wasm / 移体积”两类收益差异。

## 完成态（2026-03-02）
- M0~M3 全部完成，`agent_world_viewer` wasm 构建链路保持通过。
- wasm 主体体积：
  - baseline：`70,761,540` bytes
  - phase1(pruned)：`70,754,079` bytes
  - phase2：`48,934,067` bytes（较 phase1 减少 `21,820,012` bytes，`-30.8392%`）
- dist 口径（以 phase1/pruned 与 phase2 可比目录为准）：
  - phase1(pruned)：`70,881,376` bytes
  - phase2：`64,104,437` bytes（减少 `6,776,939` bytes，`-9.5610%`）
- 字体资源策略：
  - `ms-yahei.ttf` 不再内联进 wasm，改为运行时加载。
  - `index.html` 增加 trunk `copy-file`，只拷贝 `assets/fonts/ms-yahei.ttf`（`15,044,440` bytes）到 dist，避免回退为整包 `assets` 拷贝。
