#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Writes the ASCII STL meshes and cases.tsv for the native marked-cylinder
comparison. Coordinates are printed to six decimals so the original and the
port parse the same text; neither side computes geometry from these numbers."""
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent


def lcg(seed):
    while True:
        seed = (seed * 1664525 + 1013904223) % 2**32
        yield seed / 2**32 - 0.5


def tube(name, radius, length, segments, rings, arc_deg, axis, noise, seed):
    """A tube wall (no caps) round `axis`, closed when arc_deg is 360."""
    a = [v / math.sqrt(sum(x * x for x in axis)) for v in axis]
    trial = [1.0, 0.0, 0.0] if abs(a[0]) < 0.9 else [0.0, 1.0, 0.0]
    u = [a[1] * trial[2] - a[2] * trial[1], a[2] * trial[0] - a[0] * trial[2], a[0] * trial[1] - a[1] * trial[0]]
    n = math.sqrt(sum(x * x for x in u))
    u = [x / n for x in u]
    v = [a[1] * u[2] - a[2] * u[1], a[2] * u[0] - a[0] * u[2], a[0] * u[1] - a[1] * u[0]]
    closed = arc_deg == 360
    cols = segments if closed else segments + 1
    jitter = lcg(seed)
    pts = []
    for r in range(rings):
        t = -length / 2 + length * r / (rings - 1)
        for s in range(cols):
            th = math.radians(arc_deg) * s / segments
            rr = radius + noise * next(jitter)
            c, sn = rr * math.cos(th), rr * math.sin(th)
            pts.append(" ".join(f"{a[k] * t + u[k] * c + v[k] * sn:.6f}" for k in range(3)))
    faces = []
    for r in range(rings - 1):
        for s in range(segments):
            p = r * cols + s
            q = r * cols + (s + 1) % cols
            faces.append((pts[p], pts[q], pts[p + cols]))
            faces.append((pts[q], pts[q + cols], pts[p + cols]))
    write(name, faces)
    return rings * cols


def flat(name, n):
    pts = [f"{x} {y} 0" for y in range(n) for x in range(n)]
    faces = []
    for y in range(n - 1):
        for x in range(n - 1):
            a = y * n + x
            faces.append((pts[a], pts[a + 1], pts[a + n]))
            faces.append((pts[a + 1], pts[a + n + 1], pts[a + n]))
    write(name, faces)
    return n * n


def write(name, faces):
    body = [f"solid {name}"]
    for f in faces:
        body += ["facet normal 0 0 1", "outer loop", *(f"vertex {p}" for p in f), "endloop", "endfacet"]
    body.append("endsolid")
    (HERE / f"{name}.stl").write_text("\n".join(body) + "\n")


meshes = {
    "tube": tube("tube", 10, 20, 24, 5, 360, [0, 0, 1], 0.0, 1),
    "noisy": tube("noisy", 10, 20, 24, 6, 360, [0, 0, 1], 0.08, 7),
    "tilted": tube("tilted", 5, 12, 16, 5, 360, [1, 2, 3], 0.03, 11),
    "arc": tube("arc", 8, 10, 12, 4, 120, [0, 1, 0], 0.02, 5),
    "flat": flat("flat", 6),
    "large": flat("large", 40),
}

rows = []


def case(shape, mesh, sigma=3, units="mm"):
    rows.append((f"cyl-{len(rows):04d}", f"cyl-{shape}", units, mesh, str(meshes[mesh]), str(sigma)))


for mesh in ("tube", "noisy", "tilted", "arc"):
    for sigma in (0, 1, 2, 3):
        case("success", mesh, sigma)
for units in ("cm", "m", "in"):
    case("success", "noisy", 3, units)
for shape in (
    "assumed", "orient-xy", "orient-inplane-xy", "orient-xz", "orient-inplane-xz",
    "extend-grow", "extend-trim", "extend-zero", "extend-empty", "extend-bad-side", "extend-partial-bad",
    "name", "known", "box", "box-facing", "two", "undo", "redo", "undo-redo-twice",
):
    case(shape, "noisy")
for shape in ("orient-xy", "orient-inplane-xz", "extend-grow", "assumed"):
    case(shape, "tilted")
case("fail", "flat")
case("fail", "large")
case("fail-undo", "flat")

(HERE / "cases.tsv").write_text("".join("\t".join(r) + "\n" for r in rows))
print(f"{len(rows)} cases, meshes {meshes}")
