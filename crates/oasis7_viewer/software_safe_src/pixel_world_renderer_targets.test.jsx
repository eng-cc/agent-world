import { render, fireEvent } from '@solidjs/testing-library';
import { describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { PixelWorldRendererTargets, rendererEntityTargetStyle } from './pixel_world_renderer_targets.jsx';

describe('real renderer accessible projection', () => {
  const bounds = { width_cm: 1000, depth_cm: 1000 };
  const agent = { id: 'agent-0', pos: { x_cm: 250, y_cm: 600 } };
  it('normalizes Chinese locale aliases and does not synthesize absent renderer locations', () => {
    const view = render(() => <PixelWorldRendererTargets locale={() => 'zh-CN'} renderState={() => ({ agents: [{id:'agent-0'}], locations:[{id:'location-0',pos:{x_cm:1,y_cm:1}}] })} stageSize={() => ({width:960,height:540})} selection={() => null} onSelect={() => {}} onHover={() => {}} />);
    expect(view.getByRole('button')).toHaveAccessibleName('选择 行动体 0');
    expect(view.container.querySelector('[data-location-id]')).toBeNull();
  });
  it('centers the hit area over the camera-focused GPU agent rather than the old world-percent marker', () => {
    const style = rendererEntityTargetStyle(agent, bounds, { width: 1440, height: 900 }, { zoom: 1, pan_x_px: 350, pan_y_px: -86 });
    expect(style.left).toBe('50%');
    expect(style.top).toBe('50%');
    expect(style.transform).toBe('translate(-50%, -50%)');
    expect(style.width).toBe('44px');
  });
  it('updates pan, zoom and resize without changing selection identity', async () => {
    const [camera, setCamera] = createSignal({ zoom: 1, pan_x_px: 0, pan_y_px: 0 });
    const [size, setSize] = createSignal({ width: 1440, height: 900 });
    const select = vi.fn();
    const view = render(() => <PixelWorldRendererTargets locale={() => 'en'} renderState={() => ({ world_bounds: bounds, agents: [agent], locations: [] })} cameraState={camera} stageSize={size} selection={() => ({kind:'agent',id:'agent-0'})} onSelect={select} onHover={() => {}} />);
    const target = view.getByRole('button');
    const initial = target.style.left;
    setCamera({ zoom: 2, pan_x_px: 200, pan_y_px: -86 });
    expect(target.style.left).not.toBe(initial);
    setSize({ width: 390, height: 844 });
    expect(target.style.left).toBe(rendererEntityTargetStyle(agent,bounds,size(),camera()).left);
    await fireEvent.click(target);
    expect(select).toHaveBeenCalledWith({kind:'agent',id:'agent-0'});
    expect(view.container.querySelectorAll('[data-agent-id="agent-0"]')).toHaveLength(1);
  });
});
