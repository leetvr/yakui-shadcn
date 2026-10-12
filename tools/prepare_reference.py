#!/usr/bin/env python3
"""Build a small local reference using unmodified pinned upstream widgets/examples.
The full Next website requires the CLI/registry monorepo build. This harness isolates
actual source examples so viewport, fonts, states and theme are reproducible.
"""
import json, re, shutil, subprocess, sys
from pathlib import Path
root=Path(__file__).resolve().parent.parent
upstream=Path(sys.argv[1])
pin=json.loads((root/'reference/upstream.json').read_text())['shadcn']
assert subprocess.check_output(['git','-C',str(upstream),'rev-parse','HEAD'],text=True).strip()==pin['commit']
app=root/'tools/reference-app'
v4=upstream/'apps/v4'
src=app/'src'
for dest in ['styles/base-nova/ui','registry/bases/base/ui']:
    shutil.copytree(v4/'registry/bases/base/ui',src/dest,dirs_exist_ok=True)
# All source files remain byte-for-byte identical, including the upstream examples.
shutil.copytree(v4/'examples/base',src/'examples',dirs_exist_ok=True)
shutil.copy(v4/'registry/styles/style-nova.css',src/'nova.css')
shutil.copy(upstream/'packages/shadcn/src/tailwind.css',src/'shadcn.css')
shutil.copy(root/'crates/adequate-ui/fonts/Geist-Regular.ttf',app/'public/Geist-Regular.ttf')
shutil.copy(root/'crates/adequate-ui/fonts/Geist-Medium.ttf',app/'public/Geist-Medium.ttf')
shutil.copy(root/'crates/adequate-ui/fonts/Geist-Bold.ttf',app/'public/Geist-Bold.ttf')
# The official build replaces IconPlaceholder with the chosen library. Match Lucide.
icon=src/'app/(create)/components/icon-placeholder.tsx'
icon.parent.mkdir(parents=True,exist_ok=True)
icon.write_text('import * as React from "react"\nimport * as icons from "lucide-react"\nexport function IconPlaceholder({lucide,tabler,hugeicons,phosphor,remixicon,...props}) { const Icon=icons[lucide]; if(!Icon) throw new Error(`Missing Lucide icon ${lucide}`); return <Icon {...props}/> }\n')
theme_text=(v4/'registry/themes.ts').read_text().split('name: "stone"')[0]
blocks=re.findall(r'(light|dark): \{(.*?)\n      \}',theme_text,re.S)
css='@source "./styles";\n@source "./examples";\n@source "./registry";\n@import "tailwindcss";\n@import "tw-animate-css";\n@import "./shadcn.css";\n@import "./nova.css";\n@custom-variant dark (&:is(.dark *));\n'
css+='@theme inline {\n --font-sans: Geist;\n'
keys=[]
for mode,block in blocks:
    tokens=re.findall(r'(?:"([\w-]+)"|([\w-]+)): "([^"]+)"',block)
    if mode=='light':keys=[a or b for a,b,v in tokens]
for key in keys:
    if key!='radius':css+=f' --color-{key}: var(--{key});\n'
for name,mult in [('sm',.6),('md',.8),('lg',1),('xl',1.4),('2xl',1.8),('3xl',2.2),('4xl',2.6)]:
    css+=f' --radius-{name}: calc(var(--radius) * {mult});\n'
css+='}\n'
for mode,block in blocks:
    css+=(':root' if mode=='light' else '.dark')+' {\n'
    for a,b,v in re.findall(r'(?:"([\w-]+)"|([\w-]+)): "([^"]+)"',block):css+=f' --{a or b}: {v};\n'
    css+='}\n'
css+='''@font-face {font-family: Geist; src:url('/Geist-Regular.ttf');font-weight:400;}
@font-face {font-family: Geist; src:url('/Geist-Medium.ttf');font-weight:500;}
@font-face {font-family: Geist; src:url('/Geist-Bold.ttf');font-weight:600 900;}
body { margin:0; color:var(--foreground); background:var(--background); font-family:Geist; }
#root { min-height:100vh; display:flex;align-items:center;justify-content:center; }
* { border-color:var(--border); }
'''
(src/'theme.css').write_text(css)
print('Prepared unmodified upstream source at',src)
