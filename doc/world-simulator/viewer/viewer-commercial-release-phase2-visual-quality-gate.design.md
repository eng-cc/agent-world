# Viewer 商业化发行 Phase 2 视觉质量门禁设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase2-visual-quality-gate.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-commercial-release-phase2-visual-quality-gate.project.md`

## 1. 设计定位
定义 Viewer 商业化发行的视觉质量门禁：以 snapshot 基线、Viewer 定向测试与 wasm gate 组成可执行的 CI/本地验证链路，把“视觉不回退”变成可审计门禁。

## 2. 设计结构
- 基线脚本层：`viewer-visual-baseline.sh` 负责 snapshot 基线校验与定向测试。
- CI 接入层：`ci-tests.sh` 与 `rust.yml` 扩展 viewer 相关门禁 job。
- 轻量运行层：对 GPU 不稳定环境采用必要即跳过、文件完整性补强的策略。
- 手册同步层：`testing-manual.md` 与项目文档保持一致门禁口径。

## 3. 关键接口 / 入口
- `scripts/viewer-visual-baseline.sh`
- `scripts/ci-tests.sh`
- `.github/workflows/rust.yml`
- snapshot 基线文件
- wasm gate

## 4. 约束与边界
- 视觉门禁要尽量轻量，避免 CI 时长不可控膨胀。
- 无 GPU 环境下要保证规则稳定且可解释。
- 基线脚本、CI 和测试手册口径必须同批更新。
- 本阶段不直接提升美术质量，只建立质量闸门。

## 5. 设计演进计划
- 先建立视觉基线脚本。
- 再接入 CI required 与 wasm gate。
- 最后通过手册和状态回写把 Phase 2 固化为发布门禁。
