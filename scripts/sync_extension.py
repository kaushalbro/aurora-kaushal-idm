#!/usr/bin/env python3
"""Synchronize shared extension files without replacing browser-specific manifests."""
import argparse
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'apps/aurora-extension'
SHARED = ('background', 'content', 'core', 'offscreen', 'options', 'popup', 'pkg', 'icons')
BROWSERS = ('chrome', 'brave', 'edge', 'firefox', 'safari')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='report drift without changing files')
    args = parser.parse_args()
    drift = []
    files = [p for directory in SHARED for p in (SOURCE / directory).rglob('*') if p.is_file()]
    for browser in BROWSERS:
        destination = ROOT / f'apps/aurora-{browser}-extension'
        for source in files:
            target = destination / source.relative_to(SOURCE)
            if target.exists() and target.read_bytes() == source.read_bytes():
                continue
            if args.check:
                drift.append(str(target.relative_to(ROOT)))
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, target)
    if drift:
        raise SystemExit('Extension source drift:\n' + '\n'.join(drift))
    print('All five browser copies match the shared extension source.')


if __name__ == '__main__':
    main()
