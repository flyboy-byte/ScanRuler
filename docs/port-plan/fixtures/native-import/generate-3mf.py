#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Write the native scan.open 3MF fixtures (fixed timestamps, so reruns are byte-identical)."""
import os, struct, zipfile, zlib

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "3mf")
NS = 'xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"'
CT = ('<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
      '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
      '<Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>'
      '<Default Extension="png" ContentType="image/png"/></Types>')
RELS = ('<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
        '<Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>')
QUAD_V = [(0, 0, 0), (10, 0, 0), (10, 10, 0), (0, 10, 0)]
QUAD_T = [(0, 1, 2), (0, 2, 3)]


def mesh(v, t, extra=""):
    vs = "".join(f'<vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in v)
    ts = "".join(f'<triangle v1="{a}" v2="{b}" v3="{c}"{extra}/>' for a, b, c in t)
    return f"<mesh><vertices>{vs}</vertices><triangles>{ts}</triangles></mesh>"


def model(body, unit="millimeter", build='<build><item objectid="1"/></build>', ns=NS):
    return f'<?xml version="1.0" encoding="UTF-8"?><model unit="{unit}" {ns}><resources>{body}</resources>{build}</model>'


def png(w=2, h=2):
    raw = b"".join(b"\0" + bytes([255, 0, 0, 255] * w) for _ in range(h))
    def chunk(k, d):
        return struct.pack(">I", len(d)) + k + d + struct.pack(">I", zlib.crc32(k + d))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def package(name, model_xml, extra=(), method=zipfile.ZIP_STORED):
    with zipfile.ZipFile(os.path.join(OUT, name + ".3mf"), "w", method) as z:
        for path, data in [("[Content_Types].xml", CT), ("_rels/.rels", RELS), ("3D/3dmodel.model", model_xml), *extra]:
            info = zipfile.ZipInfo(path, (1980, 1, 1, 0, 0, 0))
            info.compress_type = method
            z.writestr(info, data)


def main():
    os.makedirs(OUT, exist_ok=True)
    obj = lambda m: f'<object id="1" type="model">{m}</object>'
    package("quad", model(obj(mesh(QUAD_V, QUAD_T))))
    package("deflated", model(obj(mesh(QUAD_V, QUAD_T))), method=zipfile.ZIP_DEFLATED)
    package("inch", model(obj(mesh(QUAD_V, QUAD_T)), unit="inch"))
    package("centimeter", model(obj(mesh(QUAD_V, QUAD_T)), unit="centimeter"))
    package("transform", model(obj(mesh(QUAD_V, QUAD_T)),
                               build='<build><item objectid="1" transform="1 0 0 0 1 0 0 0 1 5 6 7"/>'
                                     '<item objectid="1" transform="0 1 0 -1 0 0 0 0 1 30 0 0"/></build>'))
    package("components", model(obj(mesh(QUAD_V, QUAD_T)).replace('id="1"', 'id="2"')
                                + '<object id="1" type="model"><components><component objectid="2"/>'
                                  '<component objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 20"/></components></object>'))
    package("colors", model('<basematerials id="5"><base name="r" displaycolor="#FF0000"/><base name="b" displaycolor="#0000FFFF"/></basematerials>'
                            '<object id="1" type="model" pid="5" pindex="0">'
                            + mesh(QUAD_V, QUAD_T).replace('<triangle v1="0" v2="2" v3="3"/>', '<triangle v1="0" v2="2" v3="3" pid="5" p1="1"/>')
                            + "</object>"))
    tex_ns = NS + ' xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02"'
    textured = model('<m:texture2d id="7" path="/3D/Texture/skin.png" contenttype="image/png"/>'
                     '<m:texture2dgroup id="8" texid="7"><m:tex2coord u="0" v="0"/><m:tex2coord u="1" v="0"/><m:tex2coord u="1" v="1"/><m:tex2coord u="0" v="1"/></m:texture2dgroup>'
                     '<object id="1" type="model">' + mesh(QUAD_V, QUAD_T, ' pid="8" p1="0" p2="1" p3="2"') + "</object>", ns=tex_ns)
    package("texture", textured, extra=[("3D/Texture/skin.png", png())])
    with zipfile.ZipFile(os.path.join(OUT, "no-model.3mf"), "w") as z:
        z.writestr(zipfile.ZipInfo("[Content_Types].xml", (1980, 1, 1, 0, 0, 0)), CT)
    package("no-triangles", model(obj("<mesh><vertices></vertices><triangles></triangles></mesh>")))
    package("bad-index", model(obj(mesh(QUAD_V, [(0, 1, 9)]))))
    with open(os.path.join(OUT, "damaged.3mf"), "wb") as f:
        f.write(b"PK\x03\x04 this is not really a zip")


if __name__ == "__main__":
    main()
