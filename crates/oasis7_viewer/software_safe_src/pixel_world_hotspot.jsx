function isZhLocale(locale) {
  return String(locale || "").trim().toLowerCase().startsWith("zh");
}

function tr(locale, zh, en) {
  return isZhLocale(locale) ? zh : en;
}

export function pixelWorldHotspotKindLabel(locale, kind) {
  const normalizedKind = String(kind || "info").trim().toLowerCase();
  if (normalizedKind === "blocker") return tr(locale, "阻塞", "Blocker");
  if (normalizedKind === "goal") return tr(locale, "目标", "Goal");
  return tr(locale, "信息", "Info");
}

export function pixelWorldHotspotAccessibleLabel(locale, hotspot) {
  const kindLabel = pixelWorldHotspotKindLabel(locale, hotspot?.kind);
  const label = String(hotspot?.label || hotspot?.id || "").trim();
  return tr(
    locale,
    `${kindLabel}热点：${label}；只读说明。`,
    `${kindLabel} hotspot: ${label}; read-only explanation.`,
  );
}

export function pixelWorldHotspotTooltipId(hotspot) {
  const id = String(hotspot?.id || hotspot?.kind || "hotspot")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-");
  return `pixel-world-hotspot-tooltip-${id || "hotspot"}`;
}

export function PixelWorldHotspot(props) {
  const hotspot = () => props.hotspot;
  const selection = () => ({ kind: "hotspot", id: hotspot().id });
  const inspect = () => {
    props.onHotspotInspect?.(selection());
    props.onHover?.(selection());
  };
  return (
    <button
      type="button"
      class="pixel-world-hotspot"
      data-hotspot-kind={hotspot().kind}
      style={props.style}
      title={`${hotspot().kind}:${hotspot().label}`}
      aria-label={pixelWorldHotspotAccessibleLabel(props.locale, hotspot())}
      aria-describedby={pixelWorldHotspotTooltipId(hotspot())}
      onFocus={inspect}
      onBlur={() => props.onHover?.(null)}
      onMouseEnter={() => props.onHover?.(selection())}
      onMouseLeave={() => props.onHover?.(null)}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          props.onHotspotClear?.();
          props.onHover?.(null);
          event.currentTarget.blur();
        }
      }}
      onClick={(event) => {
        event.preventDefault();
        event.stopPropagation();
        inspect();
      }}
    >
      <span aria-hidden="true">{hotspot().kind === "blocker" ? "!" : hotspot().kind === "goal" ? "G" : "i"}</span>
    </button>
  );
}

export function PixelWorldHotspotTooltip(props) {
  const hotspot = () => props.hotspot;
  return (
    <div
      id={pixelWorldHotspotTooltipId(hotspot())}
      class="pixel-world-canvas__hotspot-tooltip"
      data-hotspot-tooltip
      role="status"
    >
      <span>{`${pixelWorldHotspotKindLabel(props.locale, hotspot().kind)}: ${hotspot().label}`}</span>
      <button
        type="button"
        class="pixel-world-canvas__hotspot-tooltip-close"
        aria-label={tr(props.locale, "关闭热点说明", "Close hotspot explanation")}
        onClick={(event) => {
          event.preventDefault();
          event.stopPropagation();
          props.onClose?.();
        }}
      >
        ×
      </button>
    </div>
  );
}
