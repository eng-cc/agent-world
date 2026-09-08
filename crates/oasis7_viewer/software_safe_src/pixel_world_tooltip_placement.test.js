import { describe, expect, it } from "vitest";
import { hotspotTooltipSafeBand } from "./pixel_world_tooltip_placement.js";

describe("hotspot tooltip safe band", () => {
  it.each([
    [844, 224, 420],
    [390, 137, 174.8],
    [360, 137, 171.2],
  ])("keeps summary and command actions clear at viewport height %s", (height, summaryBottom, commandTop) => {
    const band = hotspotTooltipSafeBand({ bottom: summaryBottom }, { top: commandTop }, height);
    expect(band.top).toBeGreaterThan(summaryBottom);
    expect(band.maxHeight).toBeGreaterThanOrEqual(26);
    expect(band.top + band.maxHeight).toBeLessThan(commandTop);
    expect(band.top + band.maxHeight).toBeLessThan(height);
  });
});
