#!/usr/bin/env python3
"""Validate coverage claims against public source, actual routes, tests and captures."""
import json,re,subprocess
from pathlib import Path
root=Path(__file__).resolve().parent.parent
inventory=json.loads((root/'reference/components.json').read_text())
public=set();tests=set()
for path in (root/'crates/adequate-ui').rglob('*.rs'):
 text=path.read_text()
 if '/src/' in str(path):public.update(re.findall(r'\bpub\s+(?:fn|struct|enum|type)\s+(\w+)',text))
for path in (root/'crates').rglob('*.rs'):
 tests.update(re.findall(r'#\[test\]\s*fn\s+(\w+)',path.read_text()))
listing=subprocess.check_output([str(root/'target/debug/gallery'),'--list'],text=True)
routes={line.split(': ',1)[0]:set(line.split(': ',1)[1].split(', ')) for line in listing.splitlines()}
components=inventory['components']
assert len({c['name'] for c in components})==len(components)
assert set(routes)=={c['name'] for c in components}
comparisons=0
for c in components:
 assert set(c['rust_api'])<=public,(c['name'],'unknown public API',set(c['rust_api'])-public)
 assert set(c['gallery_examples'])<=routes[c['name']],(c['name'],'unknown gallery route')
 assert all(name.split('::')[-1] in tests for name in c['behaviour_tests']),(c['name'],'unknown test')
 assert {e['name'] for e in c['example_coverage']}==set(c['upstream_examples'])
 for e in c['example_coverage']:
  if e['gallery_variant'] is not None:assert e['gallery_variant'] in routes[c['name']]
  for comparison in e['visual_comparisons']:
   path=root/comparison['path']
   assert all((path/file).is_file() for file in ['upstream.png','native.png','side-by-side.png','difference.png','metrics.json']),(e['name'],path)
   comparisons+=1
  if c['status']=='complete':
   assert e['status']=='complete' and e['review']=='accepted'
   assert {'light','dark'}<={v['theme'] for v in e['visual_comparisons']}
 if c['status']=='complete':assert c['rust_api'] and c['gallery_examples'] and not c['deviations']
print(f'{len(components)} API entries; {sum(len(c["upstream_examples"]) for c in components)} preview records; {comparisons} paired captures; {sum(c["status"]=="complete" for c in components)} certified complete components')

# Prototype readiness is a separate contract from exhaustive preview parity.
# Retain both records rather than treating successful renders as full certification.
if inventory.get('completion_scope',{}).get('status')=='ready':
 verification=json.loads((root/'reference/verification.json').read_text())
 report=verification['native_smoke']
 assert report['failures']==0
 for c in components:
  assert c['rust_api'] and c['gallery_examples'],(c['name'],'missing prototype API/example')
  assert c['name'] in report['captured_components'] and {'light','dark'}<=set(report['themes']),(c['name'],'missing native theme capture')
 print('Prototype readiness: documented APIs/routes and native captures for all catalogue entries; detailed parity gaps retained.')
