"""Explicit PostgreSQL tooling and connection settings shared by backup/restore."""
import os
import pathlib
import re
import subprocess
import urllib.parse


def tool(name):
    directory = os.environ.get('EPSX_PG_BIN')
    return str(pathlib.Path(directory) / name) if directory else name


def major(name):
    output = subprocess.check_output([tool(name), '--version'], text=True)
    return int(re.search(r'(\d+)\.', output).group(1))


def server_major(env=None, database=None):
    args = [tool('psql'), '-XAt', '-c', 'SHOW server_version_num']
    if database:
        args += ['--dbname', database]
    return int(subprocess.check_output(args, env=env, text=True).strip()) // 10000


def connection_env(url):
    parsed = urllib.parse.urlsplit(url)
    if parsed.scheme not in ('postgres', 'postgresql') or not parsed.path.strip('/'):
        raise SystemExit('Explicit PostgreSQL URL required')
    env = dict(os.environ)
    for key in ('PGDATABASE', 'PGHOST', 'PGPORT', 'PGPASSWORD', 'PGSERVICE'):
        env.pop(key, None)
    env['PGDATABASE'] = urllib.parse.unquote(parsed.path.lstrip('/'))
    for name, value in (
        ('PGHOST', parsed.hostname),
        ('PGPORT', str(parsed.port) if parsed.port else None),
        ('PGUSER', urllib.parse.unquote(parsed.username) if parsed.username else None),
        ('PGPASSWORD', urllib.parse.unquote(parsed.password) if parsed.password else None),
    ):
        if value is not None:
            env[name] = value
    for name, value in urllib.parse.parse_qsl(parsed.query):
        variable = {'host': 'PGHOST', 'port': 'PGPORT',
                    'sslmode': 'PGSSLMODE', 'sslrootcert': 'PGSSLROOTCERT',
                    'options': 'PGOPTIONS', 'connect_timeout': 'PGCONNECT_TIMEOUT'}.get(name)
        if variable is None:
            raise SystemExit('Unsupported connection URL option: ' + name)
        env[variable] = value
    return env
