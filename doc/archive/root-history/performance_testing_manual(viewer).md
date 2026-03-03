# 📘 文档一：Viewer性能测试手册

---

# 一、目标

确保游戏在以下场景下满足目标帧率：

| 平台                 | 目标FPS  | 最低可接受  |
| ------------------ | ------ | ------ |
| Web、Desktop            | 60 FPS | 30 FPS |
| High Agent Density | 60 FPS | 50 FPS |
| Stress Mode        | 30 FPS | 25 FPS |

---

# 一点五、Web 渲染性能测试前置（必须）

1. Web UI 渲染性能测试必须在 GPU 硬件加速环境执行，`SwiftShader/software rendering` 数据不可作为性能结论。
2. Playwright 采样必须使用 `open ... --headed`（默认 headless 可能回退到 CPU 软件渲染）。
3. 启动后先检查控制台 `AdapterInfo`，确认不含 `SwiftShader`，再开始采样与打分。

---

# 二、核心渲染指标

## 1️⃣ 帧时间结构（Frame Time Breakdown）

| 指标           | 说明      |
| ------------ | ------- |
| Frame Time   | 总帧时间    |
| Render Time  | GPU执行时间 |
| Prepare Time | CPU准备阶段 |
| Queue Time   | 命令提交阶段  |
| Present Time | 交换缓冲    |

---

## 2️⃣ Draw Call 指标

| 指标             | 理想值    |
| -------------- | ------ |
| Draw Calls     | < 2000 |
| Batches        | 越少越好   |
| Material 切换次数  | 尽量低    |
| Shadow Pass 次数 | 可控     |

---

## 3️⃣ GPU 资源指标

| 指标          | 说明 |
| ----------- | -- |
| VRAM 使用量    |    |
| 纹理数量        |    |
| Mesh 数量     |    |
| Instance 数量 |    |

---

# 三、Bevy 中添加性能采集

---

## 1️⃣ 开启内置诊断插件

```rust
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

app
    .add_plugins(FrameTimeDiagnosticsPlugin::default())
    .add_plugins(LogDiagnosticsPlugin::default());
```

---

## 2️⃣ 自定义 Render Stage 统计

```rust
fn render_stats_system(
    diagnostics: Res<DiagnosticsStore>,
) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.average() {
            println!("FPS: {}", value);
        }
    }
}
```

---

## 3️⃣ GPU 时间采集（高级）

需要使用 Render Graph hook 或 wgpu query set。

建议：

* 添加 GPU timestamp query
* 在 render graph pass 中插入时间采样

---

# 四、自动评分模型（渲染性能评分）

---

## 1️⃣ 基础评分公式

```
RenderScore = 
  0.4 × FPS稳定度评分 +
  0.2 × DrawCall评分 +
  0.2 × VRAM使用评分 +
  0.2 × 帧抖动评分
```

---

## 2️⃣ FPS 稳定度评分

```
FPS稳定度评分 = clamp(平均FPS / 目标FPS, 0, 1)
```

---

## 3️⃣ 帧抖动评分

```
FrameJitter = stddev(frame_time)

评分 = 1 - clamp(FrameJitter / 8ms, 0, 1)
```

---

# 五、可视化雷达图结构（渲染）

```
渲染性能雷达图 = [
  FPS稳定度,
  帧平滑度,
  DrawCall效率,
  GPU利用率,
  资源占用健康度
]
```

---

# 六、性能红线（Fail 条件）

触发以下任意情况则判定为不合格：

* 连续 5 秒 FPS < 最低可接受值
* VRAM 使用 > 90%
* Frame Time > 40ms 超过 10%

---

# 七、压测模式设计

建议添加：

| 模式            | 作用   |
| ------------- | ---- |
| Agent x10     | 极限负载 |
| Shadow Stress | 大量光源 |
| Texture Flood | 纹理极限 |

---

# 八、CI 自动检测

建议：

1. Web UI 渲染性能采样使用 GPU 硬件加速环境（非 `SwiftShader/software rendering`）
2. Playwright 采样使用 `open ... --headed`
3. 输出 JSON 性能报告
4. 比对上一个版本
5. 性能回退超过 10% 自动报警

---

# 九、LLM 场景等待窗口（避免误判）

在 `llm_bootstrap` 等 LLM 场景下，首个可观测 tick 可能明显晚于连接建立时刻。

建议在性能与可用性测试中增加等待约定：

1. `connectionStatus=connected` 后先记录 baseline tick。
2. 执行 `sendControl("play")` 后，至少等待 `12s` 再判断 tick 是否推进。
3. 若仍未推进，再执行 `seek/step` 并额外等待 `6~12s`。
4. 仍无推进再判定失败，并附带等待时长与日志证据。
