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
old = {c['name']: c for c in json.loads(old_path.read_text())['components']} if old_path.exists() else {}
entries = []
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
    entries.append(entry)
old_path.write_text(json.dumps({'upstream_commit': actual, 'components': entries}, indent=2) + '\n')
print(f'{len(entries)} catalogue entries, {sum(len(c["upstream_examples"]) for c in entries)} documented previews')
