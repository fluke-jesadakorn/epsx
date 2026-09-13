#!/usr/bin/env python3
"""Prepare only epsx_dev's catalog for a pre-promotion native binary rollback.

Prices come from the running Rust backend. No payment/history tables are changed.
Run while the promotion-aware backend is still available, then dev-control rollback.
"""
import argparse
import datetime
import json
import pathlib
import subprocess
import urllib.parse
import urllib.request

PSQL = '/opt/homebrew/opt/postgresql@14/bin/psql'


def query(sql):
    return subprocess.run(
        [PSQL, '-X', '-At', '-v', 'ON_ERROR_STOP=1', '-d', 'epsx_dev'],
        input=sql, text=True, capture_output=True, check=True,
    ).stdout.strip()


def literal(value):
    return "'" + str(value).replace("'", "''") + "'"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    rows = json.loads(query("SELECT coalesce(jsonb_agg(jsonb_build_object('id',id,'name',name,'metadata',plan_metadata)),'[]') FROM plans WHERE is_active AND is_public AND plan_metadata->'pay_use_catalog_promotion'='true'::jsonb"))
    changes = []
    for row in rows:
        prices = {}
        for token in row['metadata']['pay_prices']:
            url = 'http://127.0.0.1:8080/api/payments/pay-quote/' + row['id'] + '?' + urllib.parse.urlencode({'token': token})
            with urllib.request.urlopen(url, timeout=15) as response:
                quote = json.load(response)
            prices[token] = quote['price']
        metadata = {**row['metadata'], 'pay_base_prices': row['metadata']['pay_prices'],
                    'pay_prices': prices, 'pay_use_catalog_promotion': False}
        changes.append((row, metadata))
        print(row['name'], json.dumps(prices))
    if not args.apply:
        print('Preview only. Pass --apply before rolling back the native binaries.')
        return
    if not changes:
        print('No active promotion policies need conversion.')
        return
    backup = pathlib.Path.home() / '.config/epsx/dev/backups' / ('sales-rollback-' + datetime.datetime.now().strftime('%Y%m%d-%H%M%S'))
    backup.mkdir(mode=0o700)
    path = backup / 'catalog-before.json'
    path.write_text(json.dumps(rows, indent=2))
    path.chmod(0o600)
    sql = ['BEGIN;', 'SET LOCAL lock_timeout=\'5s\';']
    for row, metadata in changes:
        sql.append("DO $guard$ BEGIN UPDATE plans SET plan_metadata=" + literal(json.dumps(metadata)) + "::jsonb,updated_at=now(),last_modified_by='dev-sales-rollback-compatibility' WHERE id=" + literal(row['id']) + ' AND plan_metadata=' + literal(json.dumps(row['metadata'])) + "::jsonb; IF NOT FOUND THEN RAISE EXCEPTION 'Catalog changed concurrently; retry preparation'; END IF; END $guard$;")
    sql.append('COMMIT;')
    query('\n'.join(sql))
    print('Prepared dev catalog. Purchase, webhook and entitlement history are unchanged.')


if __name__ == '__main__':
    main()
