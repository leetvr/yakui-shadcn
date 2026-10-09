#!/usr/bin/env python3
"""Capture every native route and validate the gallery's catalogue coverage.
Native captures alone do not count as upstream visual comparisons.
"""
import argparse,json,subprocess,time
from pathlib import Path
root=Path(__file__).resolve().parent.parent
p=argparse.ArgumentParser()
p.add_argument('--themes',nargs='+',default=['light','dark'])
p.add_argument('--components',nargs='+')
p.add_argument('--all-variants',action='store_true')
p.add_argument('--showcases',action='store_true')
p.add_argument('--output',type=Path,default=root/'visual/local/gallery')
args=p.parse_args()
listing=subprocess.check_output([str(root/'target/debug/gallery'),'--list'],text=True)
routes={}
for line in listing.splitlines():
 name,variants=line.split(': ',1);routes[name]=variants.split(', ')
inventory=json.loads((root/'reference/components.json').read_text())
expected={c['name'] for c in inventory['components']}
assert set(routes)==expected, {'missing':sorted(expected-set(routes)), 'unexpected':sorted(set(routes)-expected)}
args.output.mkdir(parents=True,exist_ok=True)
report={'started_at':time.time(),'kind':'native-smoke-captures','upstream_comparison':False,'captures':[],'failures':[]}
for name,variants in routes.items():
 if args.components and name not in args.components:continue
 for variant in variants if args.all_variants else ['default']:
  for theme in args.themes:
   output=args.output/f'{name}-{variant}-{theme}.png'
   run=subprocess.run([str(root/'target/debug/gallery'),'--component',name,'--variant',variant,'--theme',theme,'--isolated','--width','1000','--height','800','--screenshot',str(output)],capture_output=True,text=True,timeout=60)
   if run.returncode:
    report['failures'].append({'component':name,'variant':variant,'theme':theme,'error':run.stderr})
    print(f'FAILED {name} {variant} {theme}',flush=True)
   else:report['captures'].append({'component':name,'variant':variant,'theme':theme,'path':str(output),'bytes':output.stat().st_size})
   (args.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
 print(name,flush=True)
if args.showcases:
 for name in ['dashboard','settings','checkout']:
  for theme in args.themes:
   output=args.output/f'showcase-{name}-{theme}.png'
   run=subprocess.run([str(root/'target/debug/gallery'),'--showcase',name,'--theme',theme,'--width','1200','--height','1100','--screenshot',str(output)],capture_output=True,text=True,timeout=60)
   if run.returncode:report['failures'].append({'showcase':name,'theme':theme,'error':run.stderr})
   else:report['captures'].append({'showcase':name,'theme':theme,'path':str(output),'bytes':output.stat().st_size})
   (args.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(f'{len(report["captures"])} native captures; {len(report["failures"])} failures')
raise SystemExit(bool(report['failures']))
