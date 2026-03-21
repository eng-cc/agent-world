# 客户端启动器中英文切换与必填配置校验（2026-03-02）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.project.md`

审计轮次: 5

## 1. Executive Summary
- 让打包后的桌面启动器支持中文/英文界面切换，降低不同语言用户的使用门槛。
- 在点击启动前提示必填配置项缺失或格式错误，避免启动流程在子进程阶段才失败。

## 2. User Experience & Functionality
- 改造 `crates/oasis7_client_launcher` GUI：
  - 增加语言状态与切换控件（中文/English）。
  - 将主要可见文案（标题、字段、按钮、状态、提示）纳入双语渲染。
  - 新增“启动前校验”并在 UI 内展示错误列表。
  - 当存在阻断项时禁用“启动”按钮，并给出明确提示。
- 增补 launcher 单元测试，覆盖语言解析与配置校验逻辑。

## 非目标
- 本阶段不引入外部 i18n 文件系统（如 json/yaml 资源包）。
- 不改造 `oasis7_game_launcher` CLI 的参数定义与行为。
- 不新增复杂配置向导流程（仅做必填/格式校验与提示）。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
### 语言切换
- 在 launcher 内引入 `UiLanguage`（`ZhCn`/`EnUs`）状态。
- 提供界面语言切换控件，切换后立即影响当前帧文本渲染。
- 默认语言策略：
  - 优先读取 `OASIS7_CLIENT_LAUNCHER_LANG`（`zh`/`en`）。
  - 否则根据 `LANG` 推断。
  - 都无法判断时回落到中文。

### 必填配置校验
- 新增 `collect_required_config_issues(&LaunchConfig)`，返回阻断问题列表。
- 校验项（阻断启动）：
  - `scenario` 非空。
  - `live bind` / `web bind` 为 `<host:port>` 且端口合法。
  - `viewer host` 非空，`viewer port` 为 1..=65535。
  - `viewer static dir` 非空。
  - `launcher bin` 非空。
  - 当启用链运行时时：
    - `chain status bind` 合法；
    - `chain node id` 非空；
    - `chain role` 在 `sequencer|storage|observer`；
    - `chain tick ms` 为正整数；
    - `chain validators` 格式合法（若填写）。
- UI 在按钮区上方显示双语错误清单，`启动` 按钮在存在阻断项时禁用。

## 5. Risks & Roadmap
- M1：文档建档完成（设计 + 项目管理）。
- M2：launcher 完成中英文切换。
- M3：launcher 完成必填校验提示与阻断机制，补单元测试并通过 required 测试。

### Technical Risks
- 双语文案散落在 UI 代码内可能导致后续扩展成本增加。
  - 缓解：先将语言枚举与提示枚举结构化，避免字符串硬编码到业务逻辑。
- 校验规则与底层 launcher 参数规则不一致会造成“前端放行、后端失败”。
  - 缓解：复用现有 parse 函数完成格式校验，减少规则分叉。

## 完成态（2026-03-02）
- 启动器已支持中文/English 切换，标题、字段、按钮、状态与关键提示均可双语显示。
- 启动器已支持启动前必填配置校验：
  - 配置合法时显示“可启动”提示；
  - 配置不合法时展示问题清单并禁用“启动”按钮；
  - 启动流程内部仍保留 preflight 二次校验，避免绕过 UI 触发失败启动。
- 已补齐 launcher 单元测试并通过 `test_tier_required` 回归。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.project.md`，保持原文约束语义不变。
