#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Cylinder-fit comparison inputs, independent of either implementation.

One case per line: id, op, params (comma list), positions (float32 LE hex),
normals (float32 LE hex, may be empty), indices (comma list). Deterministic.
"""
import math, os, random, struct

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "cases.tsv")


def unit(v):
    n = math.sqrt(sum(x * x for x in v))
    return [x / n for x in v]


def frame(axis):
    a = unit(axis)
    h = [1, 0, 0] if abs(a[0]) < 0.9 else [0, 1, 0]
    u = unit([a[1] * h[2] - a[2] * h[1], a[2] * h[0] - a[0] * h[2], a[0] * h[1] - a[1] * h[0]])
    v = [a[1] * u[2] - a[2] * u[1], a[2] * u[0] - a[0] * u[2], a[0] * u[1] - a[1] * u[0]]
    return a, u, v


def cylinder(rng, n, r, axis=(0, 0, 1), centre=(0, 0, 0), arc=360, length=20, noise=0.0, nnoise=0.0, outliers=0.0):
    a, u, v = frame(axis)
    p, nn = [], []
    for i in range(n):
        th = math.radians(rng.uniform(0, arc))
        t = rng.uniform(-length / 2, length / 2)
        rr = r + rng.gauss(0, noise) if noise else r
        if outliers and rng.random() < outliers:
            rr += rng.uniform(-r * 0.5, r * 0.5)
        c, s = math.cos(th), math.sin(th)
        p.append([centre[k] + t * a[k] + rr * (c * u[k] + s * v[k]) for k in range(3)])
        nv = [c * u[k] + s * v[k] for k in range(3)]
        if nnoise:
            nv = unit([x + rng.gauss(0, nnoise) for x in nv])
        nn.append(nv)
    return p, nn


def plane(rng, n, noise=0.0, nnoise=0.05):
    p = [[rng.uniform(-10, 10), rng.uniform(-10, 10), rng.gauss(0, noise) if noise else 0.0] for _ in range(n)]
    nn = [unit([rng.gauss(0, nnoise), rng.gauss(0, nnoise), 1.0]) for _ in range(n)]
    return p, nn


def hexf(pts):
    return "".join(struct.pack("<3f", *q).hex() for q in pts)


def row(cid, op, params, p, nn=None, idx=None):
    if idx is None:
        idx = list(range(len(p)))
    fmt = lambda x: repr(float(x)) if isinstance(x, float) and math.isfinite(x) else ("nan" if x != x else ("inf" if x == math.inf else ("-inf" if x == -math.inf else str(x))))
    return "\t".join([cid, op, ",".join(fmt(x) for x in params), hexf(p), hexf(nn or []), ",".join(map(str, idx))])


def main():
    rng = random.Random(20261105)
    rows = []
    add = lambda op, params, p, nn=None, idx=None: rows.append(row(f"case-{len(rows):04d}", op, params, p, nn, idx))
    shapes = []
    for r, axis, arc, noise, centre in [
        (10, (0, 0, 1), 360, 0.0, (0, 0, 0)), (10, (0, 0, 1), 360, 0.05, (1, 2, 3)),
        (25, (1, 1, 0.3), 360, 0.1, (-5, 7, 2)), (5, (0.2, 1, 0), 120, 0.02, (0, 0, 0)),
        (8, (1, 0, 0), 60, 0.02, (3, 0, 0)), (0.5, (0, 0, 1), 360, 0.002, (0, 0, 0)),
        (1000, (0, 1, 1), 40, 0.5, (0, 0, 0)), (12, (0.3, -0.4, 0.86), 200, 0.05, (1e4, -2e4, 5e3)),
    ]:
        shapes.append(cylinder(rng, 400, r, axis, centre, arc, noise=noise) + (r, unit(axis), centre))
    p_out, n_out = cylinder(rng, 400, 10, (0, 1, 0), (0, 0, 0), 360, noise=0.03, outliers=0.12)
    shapes.append((p_out, n_out, 10, [0, 1, 0], (0, 0, 0)))
    for p, nn, r, axis, centre in shapes:
        tilt = unit([axis[0] + 0.05, axis[1] - 0.03, axis[2] + 0.02])
        init = [centre[0] + 0.3, centre[1] - 0.2, centre[2], *tilt, r * 1.1]
        for k in (0.0, 3.0, 2.0):
            add("clipped", init + [k], p)
        add("onaxis", axis, p)
        add("onaxis", tilt, p)
        add("axis", [], p, nn)
        add("refine", init + [60.0], p)
        add("refine", init + [3.0], p)
    for seed, it in [(0x5eed, 256), (1, 64), (99, 256)]:
        for p, nn, r, axis, centre in shapes:
            add("ransac", [seed, it], p, nn)
    for nnoise in (0.02, 0.1):
        p, nn = cylinder(rng, 300, 15, (0.1, 0.2, 1), (2, 2, 2), 150, noise=0.05, nnoise=nnoise)
        add("ransac", [0x5eed, 256], p, nn)
        add("axis", [], p, nn)
    p, nn = plane(rng, 300, 0.01)
    add("ransac", [0x5eed, 256], p, nn)
    add("axis", [], p, nn)
    add("onaxis", [0, 0, 1], p)
    add("clipped", [0, 0, 0, 0, 0, 1, 10, 3.0], p)
    p, nn = cylinder(rng, 29, 10)
    add("ransac", [0x5eed, 256], p, nn)
    p, nn = cylinder(rng, 4, 10)
    add("onaxis", [0, 0, 1], p)
    add("axis", [], p[:2], nn[:2])
    add("refine", [0, 0, 0, 0, 0, 1, 10, 60.0], p)
    add("clipped", [0, 0, 0, 0, 0, 1, 10, 3.0], p)
    p, nn = cylinder(rng, 50, 10, noise=0.01)
    bad = [q[:] for q in p]
    bad[3] = [float("nan"), 0.0, 0.0]
    add("clipped", [0, 0, 0, 0, 0, 1, 10, 3.0], bad)
    add("onaxis", [0, 0, 1], bad)
    on_axis = p + [[0.0, 0.0, 1.0]]
    add("clipped", [0, 0, 0, 0, 0, 1, 10, 3.0], on_axis)
    add("refine", [0, 0, 0, 0, 0, 1, 10, 60.0], on_axis)
    add("clipped", [0, 0, 0, 0, 0, 1, 10, 3.0], p, idx=[i for i in range(50) if i % 3] + [1, 1, 2])
    # The circle kernel on its own: x and y as plane coordinates.
    for pts in ([[1, 0, 0], [0, 1, 0], [-1, 0, 0]], [[0, 0, 0], [1, 1, 0], [2, 2, 0]], [[0, 0, 0], [1, 0, 0]],
                [[5000 + 3 * math.cos(t / 10), 4000 + 3 * math.sin(t / 10), 0] for t in range(12)]):
        add("circle", [], pts)
    with open(OUT, "w") as f:
        f.write("\n".join(rows) + "\n")
    print(len(rows), "cases")


if __name__ == "__main__":
    main()
