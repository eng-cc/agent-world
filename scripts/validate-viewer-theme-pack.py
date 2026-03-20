#!/usr/bin/env python3
"""Validate viewer theme pack structure, texture size, and mesh vertex budgets."""

from __future__ import annotations

import argparse
import json
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Tuple


ENTITIES: Tuple[str, ...] = (
    "agent",
    "location",
    "asset",
    "power_plant",
)
TEXTURE_CHANNELS: Tuple[str, ...] = (
    "base",
    "normal",
    "metallic_roughness",
    "emissive",
)


@dataclass(frozen=True)
class ThemeValidationProfile:
    name: str
    mesh_suffix: str
    preset_files: Tuple[str, ...]
    min_texture_size: int
    max_texture_size: int
    max_total_texture_bytes: int
    min_vertices: Dict[str, int]
    max_vertices: Dict[str, int]


PROFILES: Dict[str, ThemeValidationProfile] = {
    "v1": ThemeValidationProfile(
        name="v1",
        mesh_suffix="",
        preset_files=(
            "industrial_default.env",
            "industrial_matte.env",
            "industrial_glossy.env",
        ),
        min_texture_size=256,
        max_texture_size=512,
        max_total_texture_bytes=3_500_000,
        min_vertices={
            "agent": 18,
            "location": 300,
            "asset": 16,
            "power_plant": 30,
        },
        max_vertices={
            "agent": 128,
            "location": 1200,
            "asset": 96,
            "power_plant": 256,
        },
    ),
    "v2": ThemeValidationProfile(
        name="v2",
        mesh_suffix="_v2",
        preset_files=(
            "industrial_v2_default.env",
            "industrial_v2_matte.env",
            "industrial_v2_glossy.env",
        ),
        min_texture_size=512,
        max_texture_size=1024,
        max_total_texture_bytes=12_000_000,
        min_vertices={
            "agent": 48,
            "location": 1200,
            "asset": 90,
            "power_plant": 90,
        },
        max_vertices={
            "agent": 256,
            "location": 3200,
            "asset": 240,
            "power_plant": 320,
        },
    ),
    "v3": ThemeValidationProfile(
        name="v3",
        mesh_suffix="_v3",
        preset_files=(
            "industrial_v3_default.env",
            "industrial_v3_matte.env",
            "industrial_v3_glossy.env",
        ),
        min_texture_size=768,
        max_texture_size=1024,
        max_total_texture_bytes=28_000_000,
        min_vertices={
            "agent": 300,
            "location": 2600,
            "asset": 150,
            "power_plant": 160,
        },
        max_vertices={
            "agent": 900,
            "location": 5000,
            "asset": 480,
            "power_plant": 600,
        },
    ),
}


def read_png_size(path: Path) -> Tuple[int, int]:
    data = path.read_bytes()
    if len(data) < 33:
        raise ValueError("png too short")
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("invalid png signature")
    if data[12:16] != b"IHDR":
        raise ValueError("missing IHDR chunk")
    width, height = struct.unpack(">II", data[16:24])
    return width, height


def validate_mesh(path: Path) -> Tuple[int, List[str]]:
    errors: List[str] = []
    try:
        gltf = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        return 0, [f"{path}: failed to parse gltf json ({exc})"]

    try:
        primitive = gltf["meshes"][0]["primitives"][0]
        position_accessor_index = primitive["attributes"]["POSITION"]
        vertex_count = int(gltf["accessors"][position_accessor_index]["count"])
    except Exception as exc:  # noqa: BLE001
        return 0, [f"{path}: missing POSITION accessor/count ({exc})"]

    for buffer in gltf.get("buffers", []):
        uri = buffer.get("uri")
        if not uri:
            continue
        buffer_path = path.parent / uri
        if not buffer_path.exists():
            errors.append(f"{path}: missing buffer file {uri}")

    return vertex_count, errors


def validate_theme_pack(
    theme_dir: Path,
    profile: ThemeValidationProfile,
    min_texture_size: int,
    max_texture_size: int,
    max_total_texture_bytes: int,
) -> List[str]:
    errors: List[str] = []
    mesh_dir = theme_dir / "meshes"
    texture_dir = theme_dir / "textures"
    preset_dir = theme_dir / "presets"
    total_texture_bytes = 0

    for directory in (mesh_dir, texture_dir, preset_dir):
        if not directory.exists():
            errors.append(f"missing directory: {directory}")

    for preset in profile.preset_files:
        preset_path = preset_dir / preset
        if not preset_path.exists():
            errors.append(f"missing preset file: {preset_path}")

    for entity in ENTITIES:
        mesh_name = f"{entity}_industrial{profile.mesh_suffix}.gltf"
        mesh_path = mesh_dir / mesh_name
        if not mesh_path.exists():
            errors.append(f"missing mesh file: {mesh_path}")
        else:
            vertices, mesh_errors = validate_mesh(mesh_path)
            errors.extend(mesh_errors)
            required_vertices = profile.min_vertices[entity]
            if vertices < required_vertices:
                errors.append(
                    f"{mesh_path}: vertex count {vertices} < required {required_vertices}"
                )
            max_vertices = profile.max_vertices[entity]
            if vertices > max_vertices:
                errors.append(f"{mesh_path}: vertex count {vertices} > allowed {max_vertices}")

        for channel in TEXTURE_CHANNELS:
            texture_path = texture_dir / f"{entity}_{channel}.png"
            if not texture_path.exists():
                errors.append(f"missing texture file: {texture_path}")
                continue
            try:
                width, height = read_png_size(texture_path)
            except Exception as exc:  # noqa: BLE001
                errors.append(f"{texture_path}: invalid png ({exc})")
                continue
            total_texture_bytes += texture_path.stat().st_size
            if width < min_texture_size or height < min_texture_size:
                errors.append(
                    f"{texture_path}: size {width}x{height} < {min_texture_size}x{min_texture_size}"
                )
            if width > max_texture_size or height > max_texture_size:
                errors.append(
                    f"{texture_path}: size {width}x{height} > {max_texture_size}x{max_texture_size}"
                )
            if width != height:
                errors.append(f"{texture_path}: texture must be square (got {width}x{height})")

    if max_total_texture_bytes > 0 and total_texture_bytes > max_total_texture_bytes:
        errors.append(
            f"{texture_dir}: total texture bytes {total_texture_bytes} > allowed {max_total_texture_bytes}"
        )

    return errors


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate viewer theme pack assets and presets."
    )
    parser.add_argument(
        "--theme-dir",
        default="crates/oasis7_viewer/assets/themes/industrial_v2",
        help="Theme directory to validate.",
    )
    parser.add_argument(
        "--profile",
        choices=tuple(PROFILES.keys()),
        default="v2",
        help="Validation profile (controls filename suffix and thresholds).",
    )
    parser.add_argument(
        "--min-texture-size",
        type=int,
        default=0,
        help="Optional override for minimum texture size (0 means profile default).",
    )
    parser.add_argument(
        "--max-texture-size",
        type=int,
        default=0,
        help="Optional override for maximum texture size (0 means profile default).",
    )
    parser.add_argument(
        "--max-total-texture-bytes",
        type=int,
        default=0,
        help="Optional override for max total bytes across all textures (0 means profile default).",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    profile = PROFILES[args.profile]
    min_texture_size = (
        args.min_texture_size if args.min_texture_size > 0 else profile.min_texture_size
    )
    max_texture_size = (
        args.max_texture_size if args.max_texture_size > 0 else profile.max_texture_size
    )
    max_total_texture_bytes = (
        args.max_total_texture_bytes
        if args.max_total_texture_bytes > 0
        else profile.max_total_texture_bytes
    )
    theme_dir = Path(args.theme_dir)

    errors = validate_theme_pack(
        theme_dir,
        profile,
        min_texture_size,
        max_texture_size,
        max_total_texture_bytes,
    )
    if errors:
        print("theme pack validation failed:")
        for err in errors:
            print(f"- {err}")
        return 1

    print(
        "theme pack validation passed: "
        f"{theme_dir} (profile={profile.name}, min_texture={min_texture_size}, "
        f"max_texture={max_texture_size}, max_total_texture_bytes={max_total_texture_bytes})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
