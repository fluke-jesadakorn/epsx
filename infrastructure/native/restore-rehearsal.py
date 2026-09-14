#!/usr/bin/env python3
"""Restore ONLY to new rehearsal databases and a new local object/key directory."""
import argparse, hashlib, json, pathlib, subprocess, shutil, os
from pg_tools import tool, major, server_major
p=argparse.ArgumentParser();p.add_argument('--backup',type=pathlib.Path,required=True);p.add_argument('--prefix',required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
if not a.prefix.startswith('epsx_restore_') or not a.prefix.replace('_','').isalnum():raise SystemExit('Prefix must start with epsx_restore_ and contain only letters, digits, underscores')
if a.output.exists():raise SystemExit('Rehearsal output already exists')
manifest=json.loads((a.backup/'manifest.json').read_text())
dump_major=manifest.get('pg_dump_major')
if dump_major is None or major('pg_restore')!=dump_major or server_major(database='postgres')<dump_major:
    raise SystemExit('Use matching pg_restore via EPSX_PG_BIN and a server at least as new as the backup; no destination changed')
for f in manifest['files']:
    path=(a.backup/f['path']).resolve()
    if not path.is_relative_to(a.backup.resolve()):raise SystemExit('Invalid backup path')
    with path.open('rb') as stream:digest=hashlib.file_digest(stream,'sha256').hexdigest()
    if digest!=f['sha256']:raise SystemExit('Backup checksum mismatch: '+f['path'])
os.umask(0o077);a.output.mkdir(parents=True)
for i,key in enumerate(manifest['database_families']):
    name=f'{a.prefix}_{i}'
    # createdb refuses existing names. There is no DROP, --clean or overwrite path.
    subprocess.run([tool('createdb'),name],check=True)
    subprocess.run([tool('pg_restore'),'--exit-on-error','--single-transaction','--no-owner','--no-acl','--dbname',name,str(a.backup/(key+'.dump'))],check=True)
    subprocess.run([tool('psql'),'-X','--dbname',name,'--command',"SELECT schemaname,relname,n_live_tup FROM pg_stat_user_tables ORDER BY 1,2"],check=True)
shutil.copytree(a.backup/'keys',a.output/'keys');shutil.copytree(a.backup/'minio-data',a.output/'minio-data');shutil.copytree(a.backup/'config',a.output/'config')
(a.output/'result.json').write_text(json.dumps({'backup':str(a.backup.resolve()),'database_prefix':a.prefix,'checksums_verified':True,'postgres_restore':'passed','object_copy':'passed','application_flow':'not_yet_tested'},indent=2))
print('Restore completed in isolated destinations; run application and chain reconciliation checks before cutover.')
