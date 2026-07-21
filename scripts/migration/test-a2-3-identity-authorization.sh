#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a2-3-identity-authorization.sh"
contract="$repo_root/docs/migration/contracts/a2-3-identity-authorization.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-identity-authorization.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "11 routes: 0 aligned, 1 partial, 10 blocked; 10 invariants; 20 STOP blockers" "$temp_dir/integrity.out"
grep -q "no database, Redis, JWKS, service, browser, migration, deployment" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "identity authorization self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "10 blocked routes, 1 partial route, 20 STOP blockers" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.routeCount !== 11 || report.statuses.aligned !== 0 || report.statuses.partial !== 1 || report.statuses.blocked !== 10 || report.invariantCount !== 10 || report.blockerCount !== 20 || report.executionSteps !== 12 || report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
' "$temp_dir/report-one.json"

A23_IDENTITY_IN="$contract" A23_IDENTITY_OUT="$temp_dir/promoted.json" bun -e '
const value = await Bun.file(process.env.A23_IDENTITY_IN).json();
value.routes[1].status = "aligned";
value.routes[1].blockerIds = [];
await Bun.write(process.env.A23_IDENTITY_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/promoted.json" >"$temp_dir/promoted.out" 2>&1
promoted_status=$?
set -e
[ "$promoted_status" -eq 1 ] || { cat "$temp_dir/promoted.out" >&2; exit 1; }
grep -q "route policy drifted\|route status count must remain conservative" "$temp_dir/promoted.out"

A23_IDENTITY_IN="$contract" A23_IDENTITY_OUT="$temp_dir/permission.json" bun -e '
const value = await Bun.file(process.env.A23_IDENTITY_IN).json();
value.routes.find((route) => route.id === "identity.post.users").requiredPermission = "admin:users:manage";
await Bun.write(process.env.A23_IDENTITY_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/permission.json" >"$temp_dir/permission.out" 2>&1
permission_status=$?
set -e
[ "$permission_status" -eq 1 ] || { cat "$temp_dir/permission.out" >&2; exit 1; }
grep -q "route policy drifted" "$temp_dir/permission.out"

A23_IDENTITY_IN="$contract" A23_IDENTITY_OUT="$temp_dir/stale-source.json" bun -e '
const value = await Bun.file(process.env.A23_IDENTITY_IN).json();
value.source.evidence[0].blob = "0000000000000000000000000000000000000000";
await Bun.write(process.env.A23_IDENTITY_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-source.json" >"$temp_dir/stale-source.out" 2>&1
stale_status=$?
set -e
[ "$stale_status" -eq 1 ] || { cat "$temp_dir/stale-source.out" >&2; exit 1; }
grep -q "stale source blob" "$temp_dir/stale-source.out"

A23_IDENTITY_IN="$contract" A23_IDENTITY_OUT="$temp_dir/traversal.json" bun -e '
const value = await Bun.file(process.env.A23_IDENTITY_IN).json();
value.targetEvidence[0].file = "../outside";
await Bun.write(process.env.A23_IDENTITY_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/traversal.json" >"$temp_dir/traversal.out" 2>&1
traversal_status=$?
set -e
[ "$traversal_status" -eq 1 ] || { cat "$temp_dir/traversal.out" >&2; exit 1; }
grep -q "unsafe evidence path" "$temp_dir/traversal.out"

A23_IDENTITY_IN="$contract" A23_IDENTITY_OUT="$temp_dir/blocker.json" bun -e '
const value = await Bun.file(process.env.A23_IDENTITY_IN).json();
value.blockers[0].status = "resolved";
await Bun.write(process.env.A23_IDENTITY_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/blocker.json" >"$temp_dir/blocker.out" 2>&1
blocker_status=$?
set -e
[ "$blocker_status" -eq 1 ] || { cat "$temp_dir/blocker.out" >&2; exit 1; }
grep -q "STOP blocker state changed" "$temp_dir/blocker.out"

set +e
EPSX_ENV=production "$verify" --mode integrity >"$temp_dir/production.out" 2>&1
production_status=$?
set -e
[ "$production_status" -eq 1 ] || { cat "$temp_dir/production.out" >&2; exit 1; }
grep -q "production-looking environment" "$temp_dir/production.out"

set +e
REDIS_URL=redis://example.invalid "$verify" --mode integrity >"$temp_dir/live.out" 2>&1
live_status=$?
set -e
[ "$live_status" -eq 1 ] || { cat "$temp_dir/live.out" >&2; exit 1; }
grep -q "never contacts databases, Redis, JWKS, or services" "$temp_dir/live.out"

echo "identity authorization self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, status/permission/blob/path/blocker/prod/live tamper=1)"
