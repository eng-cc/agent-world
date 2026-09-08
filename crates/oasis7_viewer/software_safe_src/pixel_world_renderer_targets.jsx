import { For } from 'solid-js';
import { toCanvasPoint } from './pixel_world_hotspot_projection.js';
import { pixelWorldVisualState } from './pixel_world_visual_clarity.jsx';
import { pixelWorldReadableAgentLabel } from './pixel_world_identity.js';
import { isLocaleZh } from './legacy_core.js';

// Mirrors the renderer's logical-canvas projection, including missing-position
// presentation. Never apply collision offsets to a true world hit target.
export function rendererEntityTargetStyle(entity, worldBounds, size, camera) {
  const { width, height } = size;
  const idLength = new TextEncoder().encode(entity.id || '').length;
  const point = toCanvasPoint(entity.pos, worldBounds, width, height, camera)
    || toCanvasPoint({ x_cm: 36 + (idLength * 29) % Math.max(40, width - 72), y_cm: 44 + (idLength * 17) % Math.max(48, height - 88) }, { width_cm: width, depth_cm: height }, width, height, camera);
  return { left: `${point.x / width * 100}%`, top: `${point.y / height * 100}%`, width: '44px', height: '44px', transform: 'translate(-50%, -50%)', display: point.x < 0 || point.y < 0 || point.x > width || point.y > height ? 'none' : undefined };
}

export function PixelWorldRendererTargets(props) {
  const state = () => pixelWorldVisualState(props.renderState());
  const isZh = () => isLocaleZh(props.locale());
  const entities = () => [...state().agents.map(entity => ({entity,kind:'agent'})), ...(state().worldBounds ? state().locations.filter(entity => entity.pos) : []).map(entity => ({entity,kind:'location'}))];
  return <For each={entities()}>{({entity,kind}) => <button type="button"
    class="pixel-world-entity pixel-world-renderer-target"
    data-agent-id={kind === 'agent' ? entity.id : undefined}
    data-location-id={kind === 'location' ? entity.id : undefined}
    data-pixel-world-agent-marker={kind === 'agent' ? 'true' : undefined}
    data-pixel-world-location-marker={kind === 'location' ? 'true' : undefined}
    data-renderer-target="true"
    data-selected={props.selection()?.kind === kind && props.selection()?.id === entity.id ? 'true' : 'false'}
    aria-pressed={props.selection()?.kind === kind && props.selection()?.id === entity.id}
    aria-label={`${isZh() ? '选择' : 'Select'} ${kind === 'agent' ? pixelWorldReadableAgentLabel(entity, entity.id, isZh()) : entity.label || entity.id}`}
    style={rendererEntityTargetStyle(entity,state().worldBounds,props.stageSize(),props.cameraState?.())}
    onClick={() => props.onSelect({kind,id:entity.id})}
    onMouseEnter={() => props.onHover({kind,id:entity.id})}
    onMouseLeave={() => props.onHover(null)} />}</For>;
}
