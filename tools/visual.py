#!/usr/bin/env python3
"""Capture real upstream/native trees and emit paired PNGs plus error measurements.
Start tools/reference-app first. Run `cargo build --workspace` before this tool.
No component is marked complete by a successful capture.
"""
import argparse,json,subprocess
from pathlib import Path
root=Path(__file__).resolve().parent.parent
p=argparse.ArgumentParser()
p.add_argument('--component',default='button')
p.add_argument('--reference-example')
p.add_argument('--variants',nargs='+',default=['default','outline','secondary','ghost','destructive','link'])
p.add_argument('--themes',nargs='+',default=['light','dark'])
p.add_argument('--states',nargs='+',default=['default','hover','focus','disabled'])
p.add_argument('--output',type=Path)
p.add_argument('--width',type=int,default=600)
p.add_argument('--height',type=int,default=400)
p.add_argument('--scale-factor',type=float,default=1,help='Physical pixels per logical pixel, shared by both renderers')
p.add_argument('--keys',default='',help='Comma-separated actual keyboard events after the initial state')
args=p.parse_args()
if args.output is None:args.output=root/'visual'/args.component
if args.component!='button' and args.variants==['default','outline','secondary','ghost','destructive','link']:args.variants=['default']
manifest_path=args.output/'manifest.json'
manifest=json.loads(manifest_path.read_text()) if manifest_path.exists() else {'captures':[]}
manifest['upstream']=json.loads((root/'reference/upstream.json').read_text())
# A component can have several source routes with different supported states.
# Replace the evidence for this exact route/state, retaining its other captures.
def state_label(state):return state+('+'+args.keys if args.keys else '')
requested={(variant,theme,state_label(state)) for variant in args.variants for theme in args.themes for state in args.states}
manifest['captures']=[capture for capture in manifest['captures'] if (capture['variant'],capture['theme'],capture['state']) not in requested]
for variant in args.variants:
 for theme in args.themes:
  for state in args.states:
   out=args.output/f'{variant}-{theme}-{state_label(state)}'
   out.mkdir(parents=True,exist_ok=True)
   reference=out/'upstream.png';native=out/'native.png'
   subprocess.run(['node',str(root/'tools/reference-app/capture.mjs'),args.reference_example or f'{args.component}-{variant if args.component=="button" or variant!="default" else "demo"}',theme,str(reference),state,str(args.width),str(args.height),args.keys,'',str(args.scale_factor)],check=True,stdout=subprocess.DEVNULL)
   command=[str(root/'target/debug/gallery'),'--component',args.component,'--variant',variant,'--theme',theme,'--state',state,'--isolated','--width',str(args.width),'--height',str(args.height),'--screenshot',str(native),'--scale-factor',str(args.scale_factor)]
   if args.keys:command+=['--keys',args.keys]
   subprocess.run(command,check=True,stdout=subprocess.DEVNULL)
   subprocess.run([str(root/'target/debug/compare'),str(reference),str(native),str(out)],check=True,stdout=subprocess.DEVNULL)
   metrics=json.loads((out/'metrics.json').read_text())
   manifest['captures'].append({'component':args.component,'variant':variant,'theme':theme,'state':state_label(state),'keys':args.keys.split(',') if args.keys else [],'scale_factor':args.scale_factor,'path':str(out.relative_to(args.output)),'metrics':metrics,'review':'pending'})
   (args.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
   print(f'{variant} {theme} {state}: RMSE {metrics["rgb_rmse"]:.2f}',flush=True)
