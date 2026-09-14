#!/usr/bin/env python3
"""Keep DX hot reload enabled while disabling its automatic Rust rebuilds.

DX 0.7 exposes this switch through its documented `p` terminal shortcut only.
A private terminal lets launchd use that switch; `r` remains an explicit action.
"""
import argparse
import errno
import fcntl
import json
import os
import pathlib
import pty
import re
import select
import signal
import socket
import struct
import sys
import termios
import time

ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07]*(?:\x07|\x1b\\)")


def control_path(service):
    return pathlib.Path.home() / '.config/epsx/dev' / f'{service}.realtime.sock'


def request(service, command):
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
        client.settimeout(5)
        client.connect(str(control_path(service)))
        client.sendall(command.encode())
        return json.loads(client.recv(4096))


def serve(service, command):
    address = control_path(service)
    address.parent.mkdir(parents=True, exist_ok=True)
    # One wrapper per existing launchd job. Never replace another live socket.
    if address.exists():
        try:
            request(service, 'status')
        except (OSError, ValueError):
            address.unlink()
        else:
            raise SystemExit(f'{service}: realtime server is already running')
    pid, terminal = pty.fork()
    if pid == 0:
        os.environ.setdefault('TERM', 'xterm-256color')
        os.execvp(command[0], command)
    fcntl.ioctl(terminal, termios.TIOCSWINSZ, struct.pack('HHHH', 40, 180, 0, 0))
    stopping = False

    def stop(*_):
        nonlocal stopping
        stopping = True

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    disabled = False
    toggled = False
    recent = ''
    deadline = time.monotonic() + 600
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    listener.bind(str(address))
    address.chmod(0o600)
    listener.listen(4)
    status = None
    try:
        while not stopping:
            if not disabled and time.monotonic() > deadline:
                raise RuntimeError('DX did not confirm automatic rebuilds are disabled')
            readable, _, _ = select.select([terminal, listener], [], [], 1)
            if terminal in readable:
                try:
                    data = os.read(terminal, 65536)
                except OSError as error:
                    if error.errno != errno.EIO:
                        raise
                    break
                if not data:
                    break
                # Answer terminal cursor queries; no terminal app is involved.
                if b'\x1b[6n' in data:
                    os.write(terminal, b'\x1b[1;1R')
                sys.stdout.buffer.write(data)
                sys.stdout.buffer.flush()
                recent = (recent + data.decode(errors='replace'))[-32768:]
                plain = ANSI.sub('', recent)
                if not toggled and 'Serving your app:' in plain:
                    os.write(terminal, b'p')
                    toggled = True
                    deadline = time.monotonic() + 30
                if not disabled and re.search(r'Automatic rebuilds are currently:\s*disabled', plain):
                    disabled = True
                    print(f'\n{service}: realtime active; automatic Rust rebuilds disabled', flush=True)
            if listener in readable:
                connection, _ = listener.accept()
                with connection:
                    connection.settimeout(2)
                    action = connection.recv(128).decode().strip()
                    result = {'service': service, 'pid': pid, 'automatic_rebuilds': not disabled}
                    if action == 'rebuild' and disabled:
                        os.write(terminal, b'r')
                        result['rebuild_requested'] = True
                    elif action != 'status':
                        result['error'] = 'Only status or an explicit rebuild is supported'
                    connection.sendall(json.dumps(result).encode())
            finished, status = os.waitpid(pid, os.WNOHANG)
            if finished:
                break
            status = None
    finally:
        listener.close()
        address.unlink(missing_ok=True)
        if status is None:
            try:
                os.killpg(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            end = time.monotonic() + 10
            while time.monotonic() < end:
                finished, status = os.waitpid(pid, os.WNOHANG)
                if finished:
                    break
                status = None
                time.sleep(0.1)
            if status is None:
                os.killpg(pid, signal.SIGKILL)
                _, status = os.waitpid(pid, 0)
        os.close(terminal)
    return 0 if stopping else os.waitstatus_to_exitcode(status)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('service', choices=['bff-frontend', 'bff-admin', 'bff-pay'])
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command in (['status'], ['rebuild']):
        print(json.dumps(request(args.service, args.command[0])))
        raise SystemExit(0)
    if not args.command:
        parser.error('DX command, status, or rebuild required')
    raise SystemExit(serve(args.service, args.command))
