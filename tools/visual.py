#!/usr/bin/env python3
"""Capture real upstream/native trees and emit paired PNGs plus error measurements.
Start tools/reference-app first. Run `cargo build --workspace` before this tool.
No component is marked complete by a successful capture.
"""
import argparse,json,subprocess
from pathlib import Path
root=Path(__file__).resolve().parent.parent
p=argparse.ArgumentParser()
p.add_argument('--variants',nargs='+',default=['default','outline','secondary','ghost','destructive','link'])
p.add_argument('--themes',nargs='+',default=['light','dark'])
p.add_argument('--states',nargs='+',default=['default','hover','focus','disabled'])
p.add_argument('--output',type=Path,default=root/'visual/button')
p.add_argument('--width',type=int,default=600)
p.add_argument('--height',type=int,default=400)
args=p.parse_args()
manifest={'upstream':json.loads((root/'reference/upstream.json').read_text()),'captures':[]}
for variant in args.variants:
 for theme in args.themes:
  for state in args.states:
   out=args.output/f'{variant}-{theme}-{state}'
   out.mkdir(parents=True,exist_ok=True)
   reference=out/'upstream.png';native=out/'native.png'
   subprocess.run(['node',str(root/'tools/reference-app/capture.mjs'),f'button-{variant}',theme,str(reference),state,str(args.width),str(args.height)],check=True,stdout=subprocess.DEVNULL)
   subprocess.run([str(root/'target/debug/gallery'),'--component','button','--variant',variant,'--theme',theme,'--state',state,'--isolated','--width',str(args.width),'--height',str(args.height),'--screenshot',str(native)],check=True,stdout=subprocess.DEVNULL)
   subprocess.run([str(root/'target/debug/compare'),str(reference),str(native),str(out)],check=True,stdout=subprocess.DEVNULL)
   metrics=json.loads((out/'metrics.json').read_text())
   manifest['captures'].append({'component':'button','variant':variant,'theme':theme,'state':state,'path':str(out.relative_to(args.output)),'metrics':metrics,'review':'pending'})
   (args.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
   print(f'{variant} {theme} {state}: RMSE {metrics["rgb_rmse"]:.2f}',flush=True)
