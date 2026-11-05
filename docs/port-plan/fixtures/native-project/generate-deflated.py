# SPDX-License-Identifier: AGPL-3.0-only
"""Authored schema-1 input, not captured reference results; deterministic ZIP DEFLATE fixture."""
import json,zipfile
from pathlib import Path
r=Path(__file__).resolve().parent
m={"app":"ScanRuler","schemaVersion":1,"appVersion":"0.9.2","name":"Authored archive","workspace":"elements",
"scan":{"fileName":"authored.ply","member":"scan.ply","appliedAlignment":None,"elements":[],"dimensions":[],"nextId":13,"nextNumber":7,"nextOfKind":{"plane":6},"nextDimensionId":1,"nextOfDimGroup":{},"settings":{"method":"gaussian","sigma":3},"selectMode":"surface","showOverlays":True,"showBackfaces":True,"sections":[],"nextSectionNumber":1},
"deviation":{"source":"reference","reference":None,"align":None,"globalAlign":None,"pairs":[],"localMaxDistance":1,"targetId":None,"targetSide":1,"targetFacingDeg":60,"mapFacingDeg":90,"targetScope":"all","scope":None,"showElement":True,"range":1,"rangeAuto":True,"maxDistance":3,"maxDistanceAuto":True,"bands":None,"tolerance":0.1,"showHistogram":False,"showNominal":True,"showScan":True,"showMap":True,"split":False,"probes":[],"nextProbeId":1},
"thickness":{"measured":False,"method":"normal","maxThickness":9.3,"maxThicknessAuto":False,"coneRays":8,"coneAngleDeg":15,"normalDeviationDeg":None,"low":0,"high":10,"scaleAuto":False,"bands":None,"limit":10,"showHistogram":False,"probes":[],"nextProbeId":1},
"flat":{"image":None,"pxPerMm":None,"calSource":"none","splitAxes":False,"edgeSensitivity":0.5,"showEdges":True,"snapToEdge":True,"showGrid":True,"elements":[],"nextId":1,"nameCounts":{},"dimensions":[],"nextDimId":1,"dimCounts":{},"datum":None,"counts":[],"nextCountId":1,"notes":[],"nextNoteId":1,"turns":0,"mirror":False,"subject":{"kind":"image"},"sheets":{}}}
with zipfile.ZipFile(r/'authored-deflated.scanruler','w') as z:
 for name,data in [('project.json',json.dumps(m,ensure_ascii=True,indent=2).encode()),('scan.ply',(r/'scan.ply').read_bytes())]:
  info=zipfile.ZipInfo(name,(2000,1,1,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;z.writestr(info,data,compresslevel=6)
