#!/usr/bin/env python3
"""Create a private backup using explicitly supplied environment; never loads checkout .env.
Run after quiescing EPSX writers for a cross-database/object-store recovery point.
"""
import argparse, hashlib, json, os, pathlib, shutil, subprocess, datetime
from pg_tools import tool, major, server_major, connection_env
p=argparse.ArgumentParser()
p.add_argument('--output',required=True,type=pathlib.Path)
p.add_argument('--writers-quiesced',action='store_true',required=True)
p.add_argument('--minio-stopped',action='store_true',required=True)
a=p.parse_args()
if a.output.exists():raise SystemExit('Choose a new backup directory')
os.umask(0o077)
keys=pathlib.Path(os.environ['EPSX_KEYS_DIR']).resolve()
if not keys.is_dir():raise SystemExit('EPSX_KEYS_DIR must point to existing persistent keys')
databases={k:os.environ[k] for k in ('DATABASE_URL','ANALYTICS_DATABASE_URL','PAYMENTS_DATABASE_URL','NOTIFICATIONS_DATABASE_URL','WALLET_DATABASE_URL','PAY_SERVICE_DATABASE_URL','SUBSCRIPTION_DATABASE_URL','ANALYTICS_SERVICE_DATABASE_URL') if os.environ.get(k)}
if not databases:raise SystemExit('No explicit database URLs configured')
dump_major=major('pg_dump')
connections={key:connection_env(url) for key,url in databases.items()}
for key,pg_env in connections.items():
    if server_major(env=pg_env)!=dump_major:
        raise SystemExit('pg_dump/server major mismatch for '+key+'; set EPSX_PG_BIN to the server-matching bin directory')
a.output.mkdir(parents=True)
for key,pg_env in connections.items():
    subprocess.run([tool('pg_dump'),'--format=custom','--no-owner','--no-acl','--file',str(a.output/(key+'.dump'))],env=pg_env,check=True)
shutil.copytree(keys,a.output/'keys',symlinks=False)
# A stopped native MinIO data directory includes versions and object metadata.
minio=pathlib.Path(os.environ['EPSX_MINIO_DATA_DIR']).resolve()
if not minio.is_dir() or a.output.resolve().is_relative_to(minio):raise SystemExit('Invalid MinIO data/backup path')
shutil.copytree(minio,a.output/'minio-data',symlinks=False)
config=pathlib.Path(os.environ['EPSX_CONFIG_DIR']).resolve()
(a.output/'config').mkdir()
for f in config.glob('*.env'):shutil.copy2(f,a.output/'config'/f.name)

files=[]
for f in sorted(a.output.rglob('*')):
    if f.is_file():
        with f.open('rb') as stream: digest=hashlib.file_digest(stream,'sha256').hexdigest()
        files.append({'path':str(f.relative_to(a.output)),'sha256':digest})
(a.output/'manifest.json').write_text(json.dumps({'created_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'pg_dump_major':dump_major,'database_families':list(databases),'files':files},indent=2))
print('Backup completed:',a.output)
