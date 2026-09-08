const FOCUSABLE = 'button, a[href], input, select, textarea, [tabindex]';

export function moveFocusFromHotspotTooltip(tooltip, backwards) {
  const trigger = [...document.querySelectorAll('.pixel-world-hotspot')]
    .find((node) => node.getAttribute('aria-describedby') === tooltip.id);
  if (!trigger) return false;
  if (backwards) {
    trigger.focus();
    return true;
  }
  const controls = [...document.querySelectorAll(FOCUSABLE)].filter((node) => {
    if (node.tabIndex < 0 || node.disabled || tooltip.contains(node)) return false;
    for (let ancestor = node; ancestor; ancestor = ancestor.parentElement) {
      const style = getComputedStyle(ancestor);
      if (ancestor.hidden || ancestor.inert || style.display === 'none' || style.visibility === 'hidden') return false;
    }
    return true;
  });
  const next = controls[controls.indexOf(trigger) + 1];
  if (!next || next === trigger) return false;
  next.focus();
  return document.activeElement === next;
}
