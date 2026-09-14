#!/usr/bin/env python3
"""Run the persistent dev chain and align restored block time with wall time."""
import json
import pathlib
import signal
import subprocess
import time
import urllib.request

ROOT = pathlib.Path.home() / '.config/epsx/dev'


def rpc(method, params):
    body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': method, 'params': params}).encode()
    request = urllib.request.Request('http://127.0.0.1:8545', data=body,
                                     headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(request, timeout=2) as response:
        result = json.load(response)
    if 'error' in result:
        raise RuntimeError(result['error'])
    return result['result']


def main():
    child = subprocess.Popen([str(ROOT / 'tools/anvil'), '--host', '127.0.0.1',
        '--port', '8545', '--chain-id', '31337', '--accounts', '0',
        '--block-time', '60', '--mixed-mining', '--state', str(ROOT / 'state/anvil.json'),
        '--state-interval', '30', '--preserve-historical-states'])
    stopping = False

    def stop(signum, _frame):
        nonlocal stopping
        stopping = True
        child.send_signal(signum)

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    try:
        deadline = time.monotonic() + 120
        while not stopping and child.poll() is None:
            try:
                chain = rpc('eth_chainId', [])
                break
            except (OSError, TimeoutError):
                if time.monotonic() >= deadline:
                    raise RuntimeError('Dev Anvil did not become ready')
                time.sleep(0.25)
        else:
            return child.wait()
        if int(chain, 16) != 31337:
            raise RuntimeError('Refusing to set time outside Anvil 31337')
        head = rpc('eth_getBlockByNumber', ['latest', False])
        timestamp = max(int(time.time()), int(head['timestamp'], 16) + 1)
        rpc('evm_setTime', [timestamp])
        rpc('evm_mine', [])
        print('Dev Anvil restored; block time synchronized', flush=True)
        return child.wait()
    finally:
        if child.poll() is None:
            child.terminate()
            child.wait(timeout=30)


if __name__ == '__main__':
    raise SystemExit(main())
