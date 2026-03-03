# Viewer 发行全覆盖验收 Gate（可用性 + 视觉 + 玩法环节）

## 目标
- 在现有 `viewer-release-qa-loop.sh` 可用性门禁基础上，新增“发行级全覆盖验收”入口，避免只验证“能跑”而忽略视觉资产与关键玩法环节。
- 覆盖三类核心风险：
  - Web 可用性风险（连接、语义控制、缩放链路）。
  - 美术资产风险（主题包结构/贴图/构图与实体通道覆盖）。
  - 玩法闭环风险（工业链、治理链、危机链、经济合约链）。
- 输出统一汇总报告，给出每个环节的通过/失败状态与证据路径。

## 范围

### In Scope
- 新增一键脚本：`scripts/viewer-release-full-coverage.sh`。
- 复用并编排现有能力：
  - `scripts/viewer-release-qa-loop.sh`（Web 语义可用性门禁）。
  - `scripts/validate-viewer-theme-pack.py`（主题包结构与质量阈值校验）。
  - `scripts/viewer-theme-pack-preview.sh`（主题多变体截图）。
  - `scripts/viewer-texture-inspector.sh`（实体纹理通道覆盖截图）。
  - `scripts/llm-longrun-stress.sh`（工业与治理/危机/经济动作覆盖门禁）。
- 扩展 `viewer-theme-pack-preview.sh`，支持 `industrial_v1`/`industrial_v2` 主题包切换。
- 更新 `testing-manual.md`，新增全覆盖验收入口与产物说明。

### Out of Scope
- 不重写 viewer 渲染管线与材质算法。
- 不新增玩法协议字段。
- 不引入 CI 强制门禁（先提供本地/Agent 一键验收入口）。

## 接口 / 数据

### 新脚本接口
- 入口：`./scripts/viewer-release-full-coverage.sh`
- 关键参数：
  - `--scenario <name>`：viewer/live 场景（默认 `llm_bootstrap`）。
  - `--theme-pack <industrial_v2|industrial_v1>`：视觉验收主题包（默认 `industrial_v2`）。
  - `--variants <list>`：主题变体集合（默认 `default,matte,glossy`）。
  - `--inspect <list>`：纹理检查实体集合（默认 `all`）。
  - `--ticks-industrial <n>` / `--ticks-gameplay <n>`：玩法覆盖运行长度。
  - `--quick`：快速模式（缩短 tick 和视觉样本规模，用于本地冒烟）。
  - `--skip-*`：按需跳过某类子门禁。

### 产物目录
- 默认根目录：`output/playwright/viewer/release_full/<timestamp>/`
- 子目录：
  - `web_qa/`：Web 语义可用性门禁报告与截图。
  - `theme_preview/`：主题变体截图。
  - `texture_inspector/`：实体纹理通道截图。
  - `gameplay_industrial/`：工业链覆盖报告。
  - `gameplay_governance/`：治理/危机/经济链覆盖报告。
- 汇总：
  - `release-full-summary-<timestamp>.md`
- 视觉抓帧状态文件：
  - `theme_preview/*/capture_status.txt`
  - `texture_inspector/*/*/capture_status.txt`
  - 关键字段：`connection_status`、`snapshot_ready`、`last_error`
  - 发布门禁要求：`connection_status=connected` 且 `snapshot_ready=1`

### 玩法环节覆盖口径
- 工业链：`harvest_radiation`、`mine_compound`、`refine_compound`、`build_factory`、`schedule_recipe`。
- 治理/危机链：`open_governance_proposal`、`cast_governance_vote`、`resolve_crisis`、`grant_meta_progress`。
- 经济链：`open_economic_contract`、`accept_economic_contract`、`settle_economic_contract`。

## 里程碑
- RFCG-0：文档建档（设计 + 项目管理）。
- RFCG-1：`viewer-theme-pack-preview.sh` 支持主题包选择。
- RFCG-2：实现 `viewer-release-full-coverage.sh` 编排脚本与汇总报告。
- RFCG-3：更新 `testing-manual.md`，补充入口与口径。
- RFCG-4：本地冒烟回归与状态收口。
- RFCG-5：修复视觉门禁假阳性（connected 硬门禁 + 抓帧链路端口就绪等待）。

## 风险
- 运行时长风险（多脚本编排）：
  - 缓解：提供 `--quick` 与 `--skip-*`。
- 玩法覆盖波动风险（LLM 动作分布随机）：
  - 缓解：治理链默认加载 `fixtures/llm_baseline/state_01`，并启用 `runtime_gameplay_preset`。
- 视觉验收口径偏差风险（主题包版本不一致）：
  - 缓解：明确 `--theme-pack`，默认对齐 `industrial_v2`。
- 子脚本失败难定位风险：
  - 缓解：汇总报告按步骤记录状态、命令与产物路径，失败后仍保留已生成证据。
- 视觉门禁假阳性风险（仅按截图存在判定）：
  - 缓解：新增 `capture_status.txt` 连通性校验，未连接或无快照时直接 fail。
