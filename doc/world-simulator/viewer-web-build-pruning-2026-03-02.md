# Viewer Web 构建体积裁剪（2026-03-02）

## 目标
- 降低 `agent_world_viewer` 的 wasm 构建产物体积，优先移除 web 端不需要参与编译的模块。
- 在不改变 native 运行行为的前提下，收敛 wasm 目标的依赖图，减少无效编译与无效链接。
- 保持 Web 闭环可用（`cargo check --target wasm32-unknown-unknown` 与 trunk release 构建通过）。

## 范围
- `crates/agent_world`：
  - `viewer` 模块按 target 进行裁剪（wasm 仅保留 client 侧协议/鉴权相关）。
  - `runtime` 与 `consensus_action_payload` 从 wasm 编译路径剥离。
  - `simulator` 在 wasm 下对“源码编译模块”动作提供明确拒绝兜底，避免 runtime 依赖回流。
  - `llm_agent` 从 wasm 编译路径剥离，同时保留 viewer UI 需要的默认 prompt 常量导出。
  - `Cargo.toml` 中 native-only 依赖下沉到 `cfg(not(target_arch = "wasm32"))`。
- `crates/agent_world_viewer`：
  - 仅做必要兼容改动（若 `agent_world::simulator` 对外导出项发生条件化变化）。

## 接口 / 数据
- 编译目标：
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer --target wasm32-unknown-unknown`
  - `trunk build --release`
- 体积对比口径：
  - 关注 `*_bg.wasm` 文件字节大小变化。
  - 辅助使用 `cargo tree --target wasm32-unknown-unknown` 观察依赖收敛。
- 行为兼容：
  - native 路径保留现有 `viewer live/server/web_bridge` 与 runtime 功能。
  - wasm 路径不支持的动作必须返回明确拒绝信息，不做 silent drop。

## 里程碑
- M0：完成建档（本文件 + `.project.md`）。
- M1：完成 wasm 路径模块裁剪与依赖下沉。
- M2：完成 wasm check + trunk release 构建与体积对比。
- M3：完成项目管理文档状态收口与开发日志记录。

## 风险
- 条件编译边界调整可能影响部分共享导出符号：
  - 通过最小兼容导出（常量保留）与定向编译回归兜底。
- runtime 依赖剥离可能牵连 simulator 内部调用链：
  - 通过 wasm 下显式拒绝分支替代 runtime 调用，避免编译失败或运行期不确定行为。
- 仅做模块裁剪后，若体积下降不达预期：
  - 记录下一阶段候选（如字体资产策略、Bevy feature 细化）。
