# SPDX-License-Identifier: AGPL-3.0-only
import hashlib,json,re,struct,unittest
from pathlib import Path

class ErrorCorpusTests(unittest.TestCase):
    def test_committed_fields_and_well_formed_json_preserve_message_units(self):
        root=Path(__file__).resolve().parents[3]
        report=json.loads((root/'docs/port-plan/evidence/2026-11-02-engine-errors.json').read_text())
        self.assertEqual(report['constructed_error_objects'],52);self.assertEqual(report['working_commands_claimed'],0)
        self.assertEqual(len(report['cases']),52)
        for case in report['cases']:
            files=case['files'];self.assertEqual(files['original.bin']['sha256'],files['rust.bin']['sha256'])
            for name,item in files.items():
                if name=='input.bin':continue
                b=bytes.fromhex(item['hex']);self.assertEqual(len(b),item['bytes'])
                self.assertEqual(hashlib.sha256(b).hexdigest(),item['sha256'])
            b=bytes.fromhex(files['original.bin']['hex']);self.assertEqual(b[:4],b'ERR1');fields=[];at=4
            for _ in range(5):
                n=struct.unpack('<I',b[at:at+4])[0];at+=4;fields.append(b[at:at+n]);at+=n
            self.assertEqual(at,len(b));code,name,msg,display,wire=fields
            self.assertEqual(name,'CommandError'.encode('utf-16-le'))
            self.assertEqual(display,name+((': '.encode('utf-16-le')+msg) if msg else b''))
            parsed=json.loads(wire);self.assertEqual(list(parsed),['code','message'])
            self.assertEqual(parsed['code'],code.decode('ascii'))
            self.assertEqual(parsed['message'].encode('utf-16-le','surrogatepass'),msg)
            self.assertEqual(files['original.stderr.txt']['bytes'],0)

    def test_fixed_hashes_all_codes_and_compiled_schedule(self):
        root=Path(__file__).resolve().parents[3];folder=root/'docs/port-plan/fixtures/engine-errors'
        manifest=json.loads((folder/'manifest.json').read_text());rows=manifest['cases'];ids=[c['id'] for c in rows]
        self.assertEqual(manifest['seeds'],[1,7,99,1039]);self.assertEqual(manifest['license'],'AGPL-3.0-only')
        self.assertEqual(manifest['reference_sha'],'2cd89c6769a10c27ee7754b477ada521c9ef5cb7')
        self.assertEqual(len(ids),52);self.assertEqual(len(set(ids)),52)
        self.assertEqual({p.stem for p in folder.glob('*.bin')},set(ids))
        table=(root/'rust/crates/port-engine/tests/support/error_cases.rs').read_text()
        self.assertEqual(re.findall(r'id: "([a-z0-9-]+)"',table),ids)
        for code in range(12):
            for kind in ['empty','escaping','unicode','surrogates']:self.assertIn(f'code-{code:02d}-{kind}',ids)
        for c in rows:
            b=(folder/(c['id']+'.bin')).read_bytes();self.assertEqual(hashlib.sha256(b).hexdigest(),c['sha256'])
            self.assertEqual(len(b),c['bytes']);self.assertLess(b[0],12)
            self.assertEqual(len(b),5+2*struct.unpack('<I',b[1:5])[0])
