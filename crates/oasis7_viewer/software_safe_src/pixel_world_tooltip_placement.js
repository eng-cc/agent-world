export function hotspotTooltipSafeBand(summary, command, viewportHeight) {
  const top = Math.max(10, summary.bottom + 4);
  const bottom = Math.min(viewportHeight - 10, command?.top ?? viewportHeight - 10) - 4;
  return { top, maxHeight: Math.max(26, bottom - top) };
}

export function installHotspotTooltipPlacement(tooltip) {
  const feed = document.querySelector('[data-viewer-overlay="feed"]');
  const command = document.querySelector('[data-viewer-overlay="next-move"]');
  const update = () => {
    for (const property of ["top", "bottom", "max-height", "overflow-y", "padding"]) tooltip.style.removeProperty(property);
    if (!feed?.open) return;
    const summary = feed.querySelector("summary");
    if (!summary) return;
    const summaryRect = summary.getBoundingClientRect();
    const rect = tooltip.getBoundingClientRect();
    if (rect.bottom <= summaryRect.top || rect.top >= summaryRect.bottom) return;
    const commandRect = command && getComputedStyle(command).display !== "none" ? command.getBoundingClientRect() : null;
    const band = hotspotTooltipSafeBand(summaryRect, commandRect, window.innerHeight);
    tooltip.style.top = `${band.top}px`;
    tooltip.style.bottom = "auto";
    tooltip.style.maxHeight = `${band.maxHeight}px`;
    tooltip.style.overflowY = "auto";
    if (band.maxHeight < 40) tooltip.style.padding = "1px 7px";
  };
  update();
  window.addEventListener("resize", update);
  feed?.addEventListener("toggle", update);
  return () => {
    window.removeEventListener("resize", update);
    feed?.removeEventListener("toggle", update);
  };
}
