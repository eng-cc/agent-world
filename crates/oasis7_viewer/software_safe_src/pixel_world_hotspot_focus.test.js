import { afterEach, describe, expect, it } from 'vitest';
import { moveFocusFromHotspotTooltip } from './pixel_world_hotspot_focus.js';

afterEach(() => { document.body.innerHTML = ''; });
describe('portaled hotspot focus order', () => {
  it('returns backward to its trigger and continues forward past hidden controls', () => {
    document.body.innerHTML = '<button class="pixel-world-hotspot" aria-describedby="explanation">Goal</button><div hidden><button>Hidden</button></div><button disabled>Disabled</button><button id="next">Next action</button><div id="explanation"><button>Close</button></div>';
    const tooltip = document.getElementById('explanation');
    expect(moveFocusFromHotspotTooltip(tooltip, true)).toBe(true);
    expect(document.activeElement.textContent).toBe('Goal');
    expect(moveFocusFromHotspotTooltip(tooltip, false)).toBe(true);
    expect(document.activeElement.id).toBe('next');
  });
});
