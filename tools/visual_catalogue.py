#!/usr/bin/env python3
"""Run the explicitly configured upstream/native comparison routes."""
import argparse,json,subprocess,sys
from pathlib import Path
root=Path(__file__).resolve().parent.parent
p=argparse.ArgumentParser();p.add_argument('--components',nargs='+');p.add_argument('--keyboard-only',action='store_true');args=p.parse_args()
for route in json.loads((root/'reference/visual-routes.json').read_text()):
 if args.components and route['component'] not in args.components:continue
 if args.keyboard_only and 'keys' not in route:continue
 command=[sys.executable,str(root/'tools/visual.py'),'--component',route['component'],'--variants',*route['variants'],'--states',*route['states']]
 if route['component']!='button':command+=['--width','1000','--height','800']
 if 'reference_example' in route:command+=['--reference-example',route['reference_example']]
 if 'keys' in route:command+=['--keys',route['keys']]
 subprocess.run(command,check=True)
