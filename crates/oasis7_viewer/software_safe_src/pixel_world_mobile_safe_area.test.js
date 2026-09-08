import { describe, expect, it } from "vitest";
import { pixelWorldMobileFocusSelectionOffset, pixelWorldMobileSelectionChipOffset, pixelWorldMobileSelectionOffset } from "./pixel_world_mobile_safe_area.js";

describe("pixel world mobile selection safe area", () => {
  it("clears the command band while preserving the Feed gap", () => {
    expect(pixelWorldMobileSelectionOffset({ markerTop: 520, markerBottom: 566, commandTop: 420, feedBottom: 146 })).toBe(-154);
    expect(pixelWorldMobileSelectionOffset({ markerTop: 267, markerBottom: 313, commandTop: 208, feedBottom: 146 })).toBe(-113);
  });

  it("moves the selected marker beside an expanded Focus HUD", () => {
    expect(pixelWorldMobileFocusSelectionOffset({ markerLeft: 178, hudRight: 300 })).toBe(130);
  });

  it("derives collapsed Feed chip clearance from the rendered bottom for every status height", () => {
    const statusHeights = [
      ["ready", 165],
      ["replay", 165],
      ["empty", 173],
      ["gap", 181],
      ["unavailable", 189],
    ];
    for (const [, feedBottom] of statusHeights) {
      expect(pixelWorldMobileSelectionChipOffset({ chipTop: 160, feedBottom })).toBe(feedBottom - 152);
    }
    expect(pixelWorldMobileSelectionChipOffset({ chipTop: 104, feedBottom: 400, feedOpen: true })).toBe(0);
  });
});
