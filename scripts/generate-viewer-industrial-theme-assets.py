#!/usr/bin/env python3
"""Generate industrial v1 theme meshes and PBR textures for agent_world_viewer.

The script uses only Python standard library so it can run in CI/dev environments
without extra image or geometry tooling.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import struct
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Sequence, Tuple


Vec2 = Tuple[float, float]
Vec3 = Tuple[float, float, float]


@dataclass
class MeshData:
    vertices: List[Vec3]
    indices: List[int]


def clamp01(value: float) -> float:
    if value < 0.0:
        return 0.0
    if value > 1.0:
        return 1.0
    return value


def mix(a: float, b: float, t: float) -> float:
    return a * (1.0 - t) + b * t


def add3(a: Vec3, b: Vec3) -> Vec3:
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def sub3(a: Vec3, b: Vec3) -> Vec3:
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def cross3(a: Vec3, b: Vec3) -> Vec3:
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def len3(v: Vec3) -> float:
    return math.sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])


def normalize3(v: Vec3) -> Vec3:
    l = len3(v)
    if l <= 1e-8:
        return (0.0, 1.0, 0.0)
    return (v[0] / l, v[1] / l, v[2] / l)


def transform_mesh(mesh: MeshData, scale: Vec3 = (1.0, 1.0, 1.0), translate: Vec3 = (0.0, 0.0, 0.0)) -> MeshData:
    return MeshData(
        vertices=[
            (
                v[0] * scale[0] + translate[0],
                v[1] * scale[1] + translate[1],
                v[2] * scale[2] + translate[2],
            )
            for v in mesh.vertices
        ],
        indices=list(mesh.indices),
    )


def merge_meshes(meshes: Sequence[MeshData]) -> MeshData:
    vertices: List[Vec3] = []
    indices: List[int] = []
    offset = 0
    for mesh in meshes:
        vertices.extend(mesh.vertices)
        indices.extend([offset + idx for idx in mesh.indices])
        offset += len(mesh.vertices)
    return MeshData(vertices=vertices, indices=indices)


def build_box(size: Vec3) -> MeshData:
    hx, hy, hz = size[0] * 0.5, size[1] * 0.5, size[2] * 0.5
    vertices = [
        (-hx, -hy, -hz),
        (hx, -hy, -hz),
        (hx, hy, -hz),
        (-hx, hy, -hz),
        (-hx, -hy, hz),
        (hx, -hy, hz),
        (hx, hy, hz),
        (-hx, hy, hz),
    ]
    indices = [
        0, 1, 2, 0, 2, 3,  # back
        4, 6, 5, 4, 7, 6,  # front
        0, 4, 5, 0, 5, 1,  # bottom
        3, 2, 6, 3, 6, 7,  # top
        0, 3, 7, 0, 7, 4,  # left
        1, 5, 6, 1, 6, 2,  # right
    ]
    return MeshData(vertices=vertices, indices=indices)


def build_octahedron(radius: float) -> MeshData:
    vertices = [
        (0.0, radius, 0.0),
        (-radius, 0.0, 0.0),
        (0.0, 0.0, radius),
        (radius, 0.0, 0.0),
        (0.0, 0.0, -radius),
        (0.0, -radius, 0.0),
    ]
    indices = [
        0, 1, 2,
        0, 2, 3,
        0, 3, 4,
        0, 4, 1,
        5, 2, 1,
        5, 3, 2,
        5, 4, 3,
        5, 1, 4,
    ]
    return MeshData(vertices=vertices, indices=indices)


def build_uv_sphere(radius: float, segments_u: int, segments_v: int) -> MeshData:
    vertices: List[Vec3] = []
    indices: List[int] = []

    for y in range(segments_v + 1):
        v = y / segments_v
        phi = math.pi * v
        for x in range(segments_u + 1):
            u = x / segments_u
            theta = (2.0 * math.pi) * u
            sx = radius * math.sin(phi) * math.cos(theta)
            sy = radius * math.cos(phi)
            sz = radius * math.sin(phi) * math.sin(theta)
            vertices.append((sx, sy, sz))

    row = segments_u + 1
    for y in range(segments_v):
        for x in range(segments_u):
            a = y * row + x
            b = a + 1
            c = a + row
            d = c + 1
            indices.extend([a, c, b, b, c, d])

    return MeshData(vertices=vertices, indices=indices)


def build_prism(radius: float, height: float, sides: int) -> MeshData:
    vertices: List[Vec3] = []
    indices: List[int] = []
    half_h = height * 0.5
    top_center = 0
    bottom_center = 1
    vertices.append((0.0, half_h, 0.0))
    vertices.append((0.0, -half_h, 0.0))

    for i in range(sides):
        ang = (2.0 * math.pi * i) / sides
        x = math.cos(ang) * radius
        z = math.sin(ang) * radius
        vertices.append((x, half_h, z))
        vertices.append((x, -half_h, z))

    for i in range(sides):
        ni = (i + 1) % sides
        top_i = 2 + i * 2
        bot_i = top_i + 1
        top_n = 2 + ni * 2
        bot_n = top_n + 1
        indices.extend([top_center, top_n, top_i])
        indices.extend([bottom_center, bot_i, bot_n])
        indices.extend([top_i, top_n, bot_i, bot_i, top_n, bot_n])

    return MeshData(vertices=vertices, indices=indices)


def compute_normals(vertices: Sequence[Vec3], indices: Sequence[int]) -> List[Vec3]:
    normals: List[Vec3] = [(0.0, 0.0, 0.0) for _ in vertices]
    for i in range(0, len(indices), 3):
        i0, i1, i2 = indices[i], indices[i + 1], indices[i + 2]
        p0, p1, p2 = vertices[i0], vertices[i1], vertices[i2]
        face = cross3(sub3(p1, p0), sub3(p2, p0))
        normals[i0] = add3(normals[i0], face)
        normals[i1] = add3(normals[i1], face)
        normals[i2] = add3(normals[i2], face)
    return [normalize3(n) for n in normals]


def compute_uvs(vertices: Sequence[Vec3]) -> List[Vec2]:
    min_y = min(v[1] for v in vertices)
    max_y = max(v[1] for v in vertices)
    span_y = max(max_y - min_y, 1e-6)
    uvs: List[Vec2] = []
    for vx, vy, vz in vertices:
        u = 0.5 + math.atan2(vz, vx) / (2.0 * math.pi)
        v = 1.0 - ((vy - min_y) / span_y)
        uvs.append((u, v))
    return uvs


def write_png(path: Path, width: int, height: int, pixel_fn) -> None:
    def chunk(chunk_type: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + chunk_type
            + data
            + struct.pack(">I", zlib.crc32(chunk_type + data) & 0xFFFFFFFF)
        )

    rows = []
    for y in range(height):
        row = bytearray([0])  # filter type 0
        for x in range(width):
            r, g, b = pixel_fn(x, y)
            row.extend((r, g, b))
        rows.append(bytes(row))

    raw = b"".join(rows)
    compressed = zlib.compress(raw, level=9)
    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", compressed)
    png += chunk(b"IEND", b"")
    path.write_bytes(png)


def hash_noise(x: int, y: int, seed: int) -> float:
    n = (x * 73856093) ^ (y * 19349663) ^ (seed * 83492791)
    n &= 0xFFFFFFFF
    n = (n ^ (n >> 13)) * 1274126177
    n &= 0xFFFFFFFF
    return (n / 0xFFFFFFFF)


def make_base_texture(path: Path, color_a: Tuple[int, int, int], color_b: Tuple[int, int, int], seed: int, size: int = 256) -> None:
    def pixel(x: int, y: int) -> Tuple[int, int, int]:
        nx = x / float(size)
        ny = y / float(size)
        stripes = 0.5 + 0.5 * math.sin((nx * 14.0 + ny * 4.0) * math.pi)
        rough_noise = hash_noise(x, y, seed)
        t = clamp01(0.6 * stripes + 0.4 * rough_noise)
        scratch = 0.18 * max(0.0, 1.0 - abs(((x + seed * 11) % 41) - 20) / 20.0)
        r = int(clamp01((mix(color_a[0], color_b[0], t) / 255.0) + scratch) * 255.0)
        g = int(clamp01((mix(color_a[1], color_b[1], t) / 255.0) + scratch) * 255.0)
        b = int(clamp01((mix(color_a[2], color_b[2], t) / 255.0) + scratch) * 255.0)
        return (r, g, b)

    write_png(path, size, size, pixel)


def make_normal_texture(path: Path, seed: int, size: int = 256) -> None:
    def pixel(x: int, y: int) -> Tuple[int, int, int]:
        nx = x / float(size)
        ny = y / float(size)
        dx = 0.14 * math.sin((nx * 16.0 + seed) * math.pi)
        dy = 0.14 * math.cos((ny * 13.0 + seed * 0.5) * math.pi)
        dx += (hash_noise(x, y, seed + 91) - 0.5) * 0.06
        dy += (hash_noise(y, x, seed + 47) - 0.5) * 0.06
        dz = math.sqrt(max(0.0, 1.0 - dx * dx - dy * dy))
        r = int((dx * 0.5 + 0.5) * 255.0)
        g = int((dy * 0.5 + 0.5) * 255.0)
        b = int((dz * 0.5 + 0.5) * 255.0)
        return (r, g, b)

    write_png(path, size, size, pixel)


def make_mr_texture(path: Path, metallic: float, roughness: float, seed: int, size: int = 256) -> None:
    def pixel(x: int, y: int) -> Tuple[int, int, int]:
        n = hash_noise(x, y, seed)
        n2 = hash_noise(x * 3, y * 3, seed + 71)
        g = int(clamp01(roughness + (n - 0.5) * 0.12) * 255.0)  # roughness in G
        b = int(clamp01(metallic + (n2 - 0.5) * 0.12) * 255.0)  # metallic in B
        return (255, g, b)

    write_png(path, size, size, pixel)


def make_emissive_texture(path: Path, emissive_color: Tuple[int, int, int], seed: int, size: int = 256) -> None:
    def pixel(x: int, y: int) -> Tuple[int, int, int]:
        line = ((x + seed * 7) % 57 == 0) or ((y + seed * 13) % 83 == 0)
        pulse = 0.5 + 0.5 * math.sin((x * 0.06 + y * 0.04 + seed) * math.pi)
        glow = 0.0
        if line:
            glow = 0.35 + 0.65 * pulse
        base = 0.03 + 0.02 * hash_noise(x, y, seed + 5)
        r = int(clamp01(base + (emissive_color[0] / 255.0) * glow) * 255.0)
        g = int(clamp01(base + (emissive_color[1] / 255.0) * glow) * 255.0)
        b = int(clamp01(base + (emissive_color[2] / 255.0) * glow) * 255.0)
        return (r, g, b)

    write_png(path, size, size, pixel)


def write_gltf(path: Path, mesh_name: str, vertices: Sequence[Vec3], indices: Sequence[int]) -> None:
    normals = compute_normals(vertices, indices)
    uvs = compute_uvs(vertices)

    buffer = bytearray()
    buffer_views = []
    accessors = []

    def append_blob(blob: bytes, target: int | None = None, align: int = 4) -> int:
        while len(buffer) % align != 0:
            buffer.append(0)
        offset = len(buffer)
        buffer.extend(blob)
        view = {"buffer": 0, "byteOffset": offset, "byteLength": len(blob)}
        if target is not None:
            view["target"] = target
        buffer_views.append(view)
        return len(buffer_views) - 1

    pos_blob = b"".join(struct.pack("<fff", *v) for v in vertices)
    nrm_blob = b"".join(struct.pack("<fff", *n) for n in normals)
    uv_blob = b"".join(struct.pack("<ff", *uv) for uv in uvs)
    idx_blob = b"".join(struct.pack("<H", idx) for idx in indices)

    pos_view = append_blob(pos_blob, target=34962)
    nrm_view = append_blob(nrm_blob, target=34962)
    uv_view = append_blob(uv_blob, target=34962)
    idx_view = append_blob(idx_blob, target=34963, align=2)

    min_pos = [min(v[i] for v in vertices) for i in range(3)]
    max_pos = [max(v[i] for v in vertices) for i in range(3)]

    accessors.append(
        {
            "bufferView": pos_view,
            "componentType": 5126,
            "count": len(vertices),
            "type": "VEC3",
            "min": min_pos,
            "max": max_pos,
        }
    )
    accessors.append(
        {
            "bufferView": nrm_view,
            "componentType": 5126,
            "count": len(normals),
            "type": "VEC3",
        }
    )
    accessors.append(
        {
            "bufferView": uv_view,
            "componentType": 5126,
            "count": len(uvs),
            "type": "VEC2",
        }
    )
    accessors.append(
        {
            "bufferView": idx_view,
            "componentType": 5123,
            "count": len(indices),
            "type": "SCALAR",
            "min": [int(min(indices))],
            "max": [int(max(indices))],
        }
    )

    bin_name = f"{path.stem}.bin"
    path.with_name(bin_name).write_bytes(buffer)

    gltf = {
        "asset": {"version": "2.0", "generator": "generate-viewer-industrial-theme-assets.py"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": f"{mesh_name}Node"}],
        "meshes": [
            {
                "name": mesh_name,
                "primitives": [
                    {
                        "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                        "indices": 3,
                    }
                ],
            }
        ],
        "buffers": [{"uri": bin_name, "byteLength": len(buffer)}],
        "bufferViews": buffer_views,
        "accessors": accessors,
    }
    path.write_text(json.dumps(gltf, indent=2) + "\n", encoding="utf-8")


def generate_meshes(mesh_dir: Path) -> None:
    agent = merge_meshes(
        [
            build_octahedron(0.56),
            transform_mesh(build_prism(0.16, 0.55, 6), translate=(0.0, 0.54, 0.0)),
        ]
    )
    location = build_uv_sphere(1.05, 24, 16)
    asset = merge_meshes(
        [
            build_box((0.95, 0.6, 0.95)),
            transform_mesh(build_box((0.78, 0.22, 0.78)), translate=(0.0, 0.41, 0.0)),
        ]
    )
    power_plant = merge_meshes(
        [
            build_prism(0.56, 1.22, 8),
            transform_mesh(build_prism(0.22, 0.72, 8), translate=(0.0, 0.74, 0.0)),
        ]
    )
    power_storage = merge_meshes(
        [
            build_prism(0.46, 1.3, 18),
            transform_mesh(build_uv_sphere(0.43, 16, 10), scale=(1.0, 0.44, 1.0), translate=(0.0, 0.65, 0.0)),
            transform_mesh(build_uv_sphere(0.43, 16, 10), scale=(1.0, 0.44, 1.0), translate=(0.0, -0.65, 0.0)),
        ]
    )

    write_gltf(mesh_dir / "agent_industrial.gltf", "AgentIndustrialMesh", agent.vertices, agent.indices)
    write_gltf(mesh_dir / "location_industrial.gltf", "LocationIndustrialMesh", location.vertices, location.indices)
    write_gltf(mesh_dir / "asset_industrial.gltf", "AssetIndustrialMesh", asset.vertices, asset.indices)
    write_gltf(mesh_dir / "power_plant_industrial.gltf", "PowerPlantIndustrialMesh", power_plant.vertices, power_plant.indices)
    write_gltf(mesh_dir / "power_storage_industrial.gltf", "PowerStorageIndustrialMesh", power_storage.vertices, power_storage.indices)


def generate_textures(texture_dir: Path) -> None:
    palette = {
        "agent": {"base_a": (84, 118, 138), "base_b": (173, 209, 228), "emissive": (72, 182, 255), "metallic": 0.58, "roughness": 0.46, "seed": 11},
        "location": {"base_a": (54, 70, 80), "base_b": (124, 149, 162), "emissive": (44, 132, 184), "metallic": 0.34, "roughness": 0.69, "seed": 23},
        "asset": {"base_a": (86, 92, 84), "base_b": (166, 156, 118), "emissive": (198, 142, 52), "metallic": 0.72, "roughness": 0.41, "seed": 31},
        "power_plant": {"base_a": (96, 74, 66), "base_b": (183, 127, 97), "emissive": (255, 122, 64), "metallic": 0.63, "roughness": 0.39, "seed": 47},
        "power_storage": {"base_a": (53, 68, 91), "base_b": (117, 142, 182), "emissive": (84, 152, 255), "metallic": 0.52, "roughness": 0.36, "seed": 59},
    }

    for entity, config in palette.items():
        make_base_texture(texture_dir / f"{entity}_base.png", config["base_a"], config["base_b"], config["seed"])
        make_normal_texture(texture_dir / f"{entity}_normal.png", config["seed"] + 5)
        make_mr_texture(
            texture_dir / f"{entity}_metallic_roughness.png",
            config["metallic"],
            config["roughness"],
            config["seed"] + 9,
        )
        make_emissive_texture(texture_dir / f"{entity}_emissive.png", config["emissive"], config["seed"] + 13)


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate industrial v1 viewer theme assets.")
    parser.add_argument(
        "--out-dir",
        default="crates/agent_world_viewer/assets/themes/industrial_v1",
        help="Output directory for generated theme assets.",
    )
    args = parser.parse_args()

    out_dir = Path(args.out_dir)
    mesh_dir = out_dir / "meshes"
    texture_dir = out_dir / "textures"
    mesh_dir.mkdir(parents=True, exist_ok=True)
    texture_dir.mkdir(parents=True, exist_ok=True)

    generate_meshes(mesh_dir)
    generate_textures(texture_dir)
    print(f"generated industrial_v1 assets under: {out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
