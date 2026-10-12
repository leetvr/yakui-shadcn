#!/usr/bin/env python3
"""Capture a deterministic native motion sequence after actual input.
Build gallery first. PNGs and timestamps always work; ffmpeg also emits a video
and contact sheet when installed. No sleeps or component-only render substitutes.
"""
import argparse
import json
import shutil
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--component', default='dialog')
parser.add_argument('--variant', default='default')
parser.add_argument('--theme', choices=['light', 'dark'], default='light')
parser.add_argument('--state', default='open')
parser.add_argument('--keys', default='')
parser.add_argument('--reference-example', help='Pinned browser harness route to compare at every sampled time')
parser.add_argument('--width', type=int, default=600)
parser.add_argument('--height', type=int, default=400)
parser.add_argument('--duration-ms', type=int, default=240)
parser.add_argument('--step-ms', type=int, default=16)
parser.add_argument('--output', type=Path)
args = parser.parse_args()
if args.step_ms <= 0 or args.duration_ms < 0:
    parser.error('step must be positive and duration nonnegative')
output = args.output or root / 'visual/local/motion' / f'{args.component}-{args.variant}-{args.theme}'
output = output.resolve()
output.mkdir(parents=True, exist_ok=True)
timestamps = list(range(0, args.duration_ms + 1, args.step_ms))
frames = []
for index, timestamp in enumerate(timestamps):
    path = output / f'{index:04d}.png'
    command = [str(root / 'target/debug/gallery'), '--component', args.component,
               '--variant', args.variant, '--theme', args.theme, '--state', args.state,
               '--isolated', '--width', str(args.width), '--height', str(args.height),
               '--animation-time-ms', str(timestamp), '--screenshot', str(path)]
    if args.keys:
        command += ['--keys', args.keys]
    subprocess.run(command, cwd=root, check=True, stdout=subprocess.DEVNULL)
    if args.reference_example:
        reference = output / f'{index:04d}-upstream.png'
        subprocess.run(['node', str(root / 'tools/reference-app/capture.mjs'),
                        args.reference_example, args.theme, str(reference), args.state,
                        str(args.width), str(args.height), args.keys, str(timestamp)],
                       cwd=root, check=True, stdout=subprocess.DEVNULL)
        subprocess.run([str(root / 'target/debug/compare'), str(reference), str(path),
                        str(output / f'{index:04d}-comparison')], cwd=root,
                       check=True, stdout=subprocess.DEVNULL)
    frames.append({'time_ms': timestamp, 'path': path.name})
manifest = {'component': args.component, 'variant': args.variant, 'theme': args.theme,
            'state': args.state, 'reference_example': args.reference_example, 'keys': args.keys.split(',') if args.keys else [],
            'viewport': [args.width, args.height], 'step_ms': args.step_ms,
            'frames': frames, 'pixels_changed': (output / frames[0]['path']).read_bytes() != (output / frames[-1]['path']).read_bytes(),
            'shadcn': json.loads((root / 'reference/upstream.json').read_text())['shadcn'],
            'yakui': json.loads((root / 'reference/upstream.json').read_text())['yakui']}
(output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
ffmpeg = shutil.which('ffmpeg')
if ffmpeg:
    source = [ffmpeg, '-hide_banner', '-loglevel', 'error', '-y', '-framerate', str(1000 / args.step_ms), '-i', str(output / '%04d.png')]
    subprocess.run(source + ['-vf', 'pad=ceil(iw/2)*2:ceil(ih/2)*2', '-c:v', 'libx264', '-pix_fmt', 'yuv420p', str(output / 'motion.mp4')], check=True)
    # Eight equally spaced samples make opening/closing direction and layout
    # changes easy to inspect alongside the full-rate native video.
    count = min(8, len(frames))
    indices = [round(i * (len(frames) - 1) / max(1, count - 1)) for i in range(count)]
    select = '+'.join(f'eq(n\\,{index})' for index in indices)
    columns = min(4, count)
    rows = (count + columns - 1) // columns
    subprocess.run(source + ['-vf', f'select={select},tile={columns}x{rows}', '-frames:v', '1', str(output / 'contact.png')], check=True)
print(f'{len(frames)} native frames; pixels changed: {manifest["pixels_changed"]}; {output}')
