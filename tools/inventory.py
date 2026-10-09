#!/usr/bin/env python3
"""Extract the pinned docs catalogue. Preserve hand-recorded verification evidence."""
import argparse, json, re, subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
p = argparse.ArgumentParser()
p.add_argument('upstream', type=Path)
args = p.parse_args()
pin = json.loads((ROOT / 'reference/upstream.json').read_text())['shadcn']
actual = subprocess.check_output(['git', '-C', str(args.upstream), 'rev-parse', 'HEAD'], text=True).strip()
if actual != pin['commit']:
    p.error(f'Expected {pin["commit"]}, got {actual}')
docs = args.upstream / pin['docs']
source = args.upstream / pin['source']
old_path = ROOT / 'reference/components.json'
prior_inventory = json.loads(old_path.read_text()) if old_path.exists() else {}
old = {c['name']: c for c in prior_inventory.get('components', [])}
entries = []
visual_routes = json.loads((ROOT / 'reference/visual-routes.json').read_text())

for name in json.loads((docs / 'meta.json').read_text())['pages']:
    path = docs / f'{name}.mdx'
    text = path.read_text() if path.exists() else ''
    examples = list(dict.fromkeys(re.findall(r'<(?:ComponentPreview|ComponentExample)\b[^>]*?\bname="([^"]+)"', text, re.S)))
    headings = re.findall(r'^## (.+)$', text, re.M)
    api = source / 'ui' / f'{name}.tsx'
    variants = []
    if api.exists():
        variants = sorted(set(re.findall(r'"cn-[\w-]+-(?:variant|size)-([\w-]+)"', api.read_text())))
    entry = {
        'name': name, 'title': re.search(r'^title: (.+)$', text, re.M).group(1) if text else name.title(),
        'description': re.search(r'^description: (.+)$', text, re.M).group(1) if text else '',
        'upstream_docs': str(path.relative_to(args.upstream)) if path.exists() else None,
        'upstream_source': str(api.relative_to(args.upstream)) if api.exists() else None,
        'documented_sections': headings, 'variants': variants, 'upstream_examples': examples,
        'rust_api': [], 'gallery_examples': [], 'behaviour_tests': [],
        'visual_comparisons': [], 'status': 'pending', 'deviations': [],
    }
    if not text:
        entry['deviations'] = ['Listed in upstream navigation, but no Base UI documentation page at pinned revision.']
    for key in ('rust_api', 'gallery_examples', 'behaviour_tests', 'visual_comparisons', 'status', 'deviations'):
        if name in old:
            entry[key] = old[name][key]
    prior_examples = {e['name']:e for e in old.get(name,{}).get('example_coverage',[])}
    coverage = []
    for preview in examples:
        example_path = args.upstream / 'apps/v4/examples/base' / f'{preview}.tsx'
        example_text = example_path.read_text() if example_path.exists() else ''
        imports = re.findall(r'from "([^\"]+)"',example_text)
        item = {
            'name':preview,
            'upstream_source':str(example_path.relative_to(args.upstream)) if example_path.exists() else None,
            'upstream_styles':sorted(set(re.findall(r'@/styles/([^/]+)/',example_text))),
            'upstream_imports':imports,
            'gallery_variant':None, 'behaviour_tests':[], 'visual_comparisons':[],
            'status':'pending', 'review':'pending',
        }
        for key in ('gallery_variant','behaviour_tests','visual_comparisons','status','review'):
            if key in prior_examples.get(preview,{}):item[key]=prior_examples[preview][key]
        for route in visual_routes:
            if route['component']!=name:continue
            for variant in route['variants']:
                reference = route.get('reference_example') or f'{name}-{variant if name=="button" or variant!="default" else "demo"}'
                if reference!=preview:continue
                item['gallery_variant']=variant
                manifest_path=ROOT/'visual'/name/'manifest.json'
                if manifest_path.exists():
                    manifest=json.loads(manifest_path.read_text())
                    item['visual_comparisons']=[{'theme':capture['theme'],'state':capture['state'],'path':f'visual/{name}/{capture["path"]}','review':capture['review']} for capture in manifest['captures'] if capture['variant']==variant]
                    item['status']='in_progress'
        coverage.append(item)
    entry['example_coverage']=coverage
    entry['visual_comparisons']=[capture['path'] for e in coverage for capture in e['visual_comparisons']]
    entries.append(entry)
old_path.write_text(json.dumps({'schema_version':2,'upstream_commit': actual, 'completion_scope': prior_inventory.get('completion_scope', {'target':'internal_prototypes','status':'in_progress','exhaustive_upstream_preview_parity_required':False}), 'components': entries}, indent=2) + '\n')
print(f'{len(entries)} catalogue entries, {sum(len(c["upstream_examples"]) for c in entries)} documented previews')
