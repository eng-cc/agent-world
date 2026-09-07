import { fireEvent, render, screen } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { PixelWorldHotspot, PixelWorldHotspotTooltip } from "./pixel_world_hotspot.jsx";

const hotspot = { id: "hotspot-blocker", kind: "blocker", label: "Blocked route" };

describe("pixel world hotspot controls", () => {
  it("is touch and keyboard inspectable without selecting or executing gameplay", () => {
    const onHover = vi.fn();
    const onHotspotInspect = vi.fn();
    const onHotspotClear = vi.fn();
    render(() => (
      <PixelWorldHotspot
        locale="en"
        hotspot={hotspot}
        style={{ left: "20%", top: "20%" }}
        onHover={onHover}
        onHotspotInspect={onHotspotInspect}
        onHotspotClear={onHotspotClear}
      />
    ));
    const marker = screen.getByRole("button", { name: /Blocker hotspot: Blocked route/i });
    expect(marker).toHaveAttribute("type", "button");
    fireEvent.focus(marker);
    fireEvent.click(marker);
    expect(onHotspotInspect).toHaveBeenCalledWith({ kind: "hotspot", id: hotspot.id });
    expect(onHover).toHaveBeenCalledWith({ kind: "hotspot", id: hotspot.id });
    expect(onHover).not.toHaveBeenCalledWith({ kind: "agent", id: hotspot.id });
    fireEvent.keyDown(marker, { key: "Escape" });
    expect(onHotspotClear).toHaveBeenCalledTimes(1);
  });

  it("keeps the explanation close control keyboard reachable and read-only", () => {
    const onClose = vi.fn();
    render(() => <PixelWorldHotspotTooltip locale="en" hotspot={hotspot} onClose={onClose} />);
    expect(screen.getByRole("status")).toHaveTextContent("Blocker: Blocked route");
    const close = screen.getByRole("button", { name: "Close hotspot explanation" });
    fireEvent.click(close);
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
