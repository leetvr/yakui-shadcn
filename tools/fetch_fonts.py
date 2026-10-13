#!/usr/bin/env python3
"""Restore pinned OFL fallback assets, verifying hashes before writing."""
import hashlib,json,urllib.request
from pathlib import Path
root=Path(__file__).resolve().parent.parent/'crates/shadcn/fonts'
manifest=json.loads((root/'noto-provenance.json').read_text())
for asset in [*manifest['assets'],manifest['license']]:
    path=root/asset['file']
    if path.exists() and hashlib.sha256(path.read_bytes()).hexdigest()==asset['sha256']:
        continue
    data=urllib.request.urlopen(asset['source']).read()
    assert hashlib.sha256(data).hexdigest()==asset['sha256'],asset['file']
    path.write_bytes(data)
print('Pinned Noto font hashes verified')
