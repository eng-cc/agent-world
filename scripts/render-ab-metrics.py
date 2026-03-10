#!/usr/bin/env python3
import datetime as dt
import json
import pathlib
import sys

run_id, url, initial_raw, final_raw, play_raw, pause_raw, step1_raw, step2_raw, nop_raw, metrics_path_raw, summary_path_raw, card_path_raw = sys.argv[1:13]
initial = json.loads(initial_raw)
final = json.loads(final_raw)
play = json.loads(play_raw)
pause = json.loads(pause_raw)
step1 = json.loads(step1_raw)
step2 = json.loads(step2_raw)
no_progress = json.loads(nop_raw)
metrics_path = pathlib.Path(metrics_path_raw)
summary_path = pathlib.Path(summary_path_raw)
card_path = pathlib.Path(card_path_raw)

commands = [play, pause, step1, step2]
expected = [item for item in commands if item.get('expectProgress')]
progressed = [item for item in expected if item.get('progressed')]
accepted = [item for item in commands if item.get('accepted')]
result = {
    'runId': run_id,
    'url': url,
    'states': {
        'initial': initial,
        'final': final,
    },
    'phases': {
        'phaseA': {
            'play': play,
            'pause': pause,
            'noProgressObservation': no_progress,
            'pass': bool(play.get('progressed') and pause.get('accepted')),
        },
        'phaseB': {
            'stepPrimary': step1,
            'stepFollowup': step2,
            'pass': bool(step1.get('progressed') and step2.get('progressed')),
        },
    },
    'commands': commands,
    'metrics': {
        'ttfcMs': play.get('firstProgressMs'),
        'totalControls': len(commands),
        'acceptedControls': len(accepted),
        'expectedProgressControls': len(expected),
        'effectiveProgressControls': len(progressed),
        'effectiveControlHitRate': (len(progressed) / len(expected)) if expected else 0,
        'maxNoProgressWindowMs': int(no_progress.get('maxNoProgressWindowMs', 0)),
        'initialTick': int(float(initial.get('tick', 0) or 0)),
        'finalTick': int(float(final.get('tick', 0) or 0)),
        'initialEventSeq': int(float(initial.get('eventSeq', 0) or 0)),
        'finalEventSeq': int(float(final.get('eventSeq', 0) or 0)),
    },
}
metrics = result.get('metrics', {})
phases = result.get('phases', {})
ttfc = metrics.get('ttfcMs')
effective = metrics.get('effectiveControlHitRate', 0)
effective_pct = f"{effective * 100:.1f}%"
expected_progress = int(metrics.get('expectedProgressControls', 0))
effective_progress = int(metrics.get('effectiveProgressControls', 0))
accepted_count = int(metrics.get('acceptedControls', 0))
total = int(metrics.get('totalControls', 0))
max_stall = int(metrics.get('maxNoProgressWindowMs', 0))
initial_tick = int(metrics.get('initialTick', 0))
final_tick = int(metrics.get('finalTick', 0))
initial_event_seq = int(metrics.get('initialEventSeq', 0))
final_event_seq = int(metrics.get('finalEventSeq', 0))
phase_a = phases.get('phaseA', {})
phase_b = phases.get('phaseB', {})
phase_a_pass = bool(phase_a.get('pass'))
phase_b_pass = bool(phase_b.get('pass'))
cmd_lines = []
for item in commands:
    cmd_lines.append(
        f"- `{item.get('name')}` action=`{item.get('action')}` "
        f"accepted={item.get('accepted')} progressed={item.get('progressed')} "
        f"category=`{item.get('failCategory')}` "
        f"tick `{item.get('beforeTick')}` -> `{item.get('afterTick')}` "
        f"eventSeq `{item.get('beforeEventSeq')}` -> `{item.get('afterEventSeq')}` "
        f"stage=`{item.get('feedbackStage')}` reason=`{item.get('feedbackReason')}`"
    )
summary_lines = [
    '# Playability A/B Metrics',
    '',
    f"- Timestamp: {dt.datetime.now().astimezone().strftime('%Y-%m-%d %H:%M:%S %Z')}",
    f"- Run ID: `{result.get('runId')}`",
    f"- URL: `{result.get('url')}`",
    '',
    '## Quant Metrics',
    f"- TTFC(ms): `{ttfc}`" if ttfc is not None else '- TTFC(ms): `null`',
    f"- Effective control hit-rate: `{effective_progress}/{expected_progress}` (`{effective_pct}`)",
    f"- Accepted control rate: `{accepted_count}/{total}`",
    f"- Max no-progress window(ms): `{max_stall}`",
    f"- Tick: `{initial_tick}` -> `{final_tick}`",
    f"- EventSeq: `{initial_event_seq}` -> `{final_event_seq}`",
    '',
    '## A/B Verdict',
    f"- A (play/pause): `{'PASS' if phase_a_pass else 'FAIL'}`",
    f"- B (step chain): `{'PASS' if phase_b_pass else 'FAIL'}`",
    f"- B step primary category: `{phase_b.get('stepPrimary', {}).get('failCategory')}`",
    f"- B step followup category: `{phase_b.get('stepFollowup', {}).get('failCategory')}`",
    '',
    '## Control Probes',
] + cmd_lines + ['']
card_lines = [
    '# 量化指标（自动填写）',
    '',
    f"- TTFC（首次可控时间，ms）：`{ttfc}`" if ttfc is not None else '- TTFC（首次可控时间，ms）：`null`',
    f"- 有效控制命中率（有效推进控制次数 / 预期推进控制次数）：`{effective_progress}/{expected_progress}`（`{effective_pct}`）",
    f"- 无进展窗口时长（ms，connected 下 tick 不变最长窗口）：`{max_stall}`",
    '- A/B 分段结论：',
    f"  - A（play/pause）：`{'PASS' if phase_a_pass else 'FAIL'}`",
    f"  - B（step链路）：`{'PASS' if phase_b_pass else 'FAIL'}`",
    f"  - B primary 失败分类：`{phase_b.get('stepPrimary', {}).get('failCategory')}`",
    f"  - B followup 失败分类：`{phase_b.get('stepFollowup', {}).get('failCategory')}`",
    '',
]
metrics_path.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
summary_path.write_text('\n'.join(summary_lines), encoding='utf-8')
card_path.write_text('\n'.join(card_lines), encoding='utf-8')
print(json.dumps(result, ensure_ascii=False))
