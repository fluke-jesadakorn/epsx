#!/usr/bin/env python3
"""Reject renderer feature leakage that corrupts Dioxus hydration transport.

Uses Cargo's resolved graph for each deployment target, not manifest text.
Run independently of workspace --all-features, which intentionally unifies
mutually exclusive renderer features and cannot validate a deployment build.
"""
import subprocess

PACKAGES = ('epsx-frontend', 'epsx-admin', 'epsx-pay-bff')


def main():
    host = next(line.split(': ', 1)[1] for line in subprocess.check_output(
        ['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
    failures = []
    for package in PACKAGES:
        for feature, target, forbidden in [('server', host, 'web'), ('web', 'wasm32-unknown-unknown', 'server')]:
            result = subprocess.check_output([
                'cargo', 'tree', '--locked', '-p', package, '--no-default-features',
                '--features', feature, '--target', target, '-i', 'dioxus-fullstack-core',
                '--depth', '0', '--format', '{p}|{f}',
            ], text=True)
            line = next(line for line in result.splitlines() if line.startswith('dioxus-fullstack-core '))
            features = set(line.split('|', 1)[1].split(','))
            if feature not in features or forbidden in features:
                failures.append(f'{package}/{target}: expected {feature} without {forbidden}, got {features}')
            print(f'{package}/{target}: {",".join(sorted(features))}')
    if failures:
        raise SystemExit('\n'.join(failures))
    print('Dioxus server/client transport features are isolated for all three apps.')


if __name__ == '__main__':
    main()
