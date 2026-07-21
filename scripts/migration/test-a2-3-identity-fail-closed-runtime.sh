#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a2-3-identity-fail-closed-runtime.sh"
contract="$repo_root/docs/migration/contracts/a2-3-identity-fail-closed-runtime.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-identity-fail-closed.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

runtime_root="$temp_dir/runtime"
mkdir -p "$runtime_root/services/identity/src"
cp "$repo_root/services/identity/src/lib.rs" "$runtime_root/services/identity/src/lib.rs"
cp "$repo_root/services/identity/src/main.rs" "$runtime_root/services/identity/src/main.rs"
cp "$repo_root/services/identity/Cargo.toml" "$runtime_root/services/identity/Cargo.toml"

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "11 boundaries aligned; functionality 1 aligned, 10 blocked; 12 STOP blockers" "$temp_dir/integrity.out"
grep -q "no database, Redis, JWKS, service, migration, or deployment executed" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || { cat "$temp_dir/readiness.out" >&2; exit 1; }
grep -q "10 functional routes blocked; 12 residual STOP blockers" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.runtimePins !== 3 || report.routerPaths !== 8 || report.routes !== 11 || report.boundaryAligned !== 11 || report.functionalityAligned !== 1 || report.functionalityBlocked !== 10 || report.invariants !== 10 || report.removedUnsafeStartup !== 5 || report.stopBlockers !== 12 || report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
' "$temp_dir/report-one.json"

A23I_IN="$contract" A23I_OUT="$temp_dir/promoted.json" bun -e '
const value = await Bun.file(process.env.A23I_IN).json();
value.productionReady = true;
value.readinessExit = 0;
await Bun.write(process.env.A23I_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/promoted.json" >"$temp_dir/promoted.out" 2>&1
promoted_status=$?
set -e
[ "$promoted_status" -eq 1 ] || { cat "$temp_dir/promoted.out" >&2; exit 1; }
grep -q "readiness sentinel changed" "$temp_dir/promoted.out"

A23I_IN="$contract" A23I_OUT="$temp_dir/permission.json" bun -e '
const value = await Bun.file(process.env.A23I_IN).json();
value.routes.find((route) => route.id === "identity.post.users").permission = "admin:users:manage";
await Bun.write(process.env.A23I_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/permission.json" >"$temp_dir/permission.out" 2>&1
permission_status=$?
set -e
[ "$permission_status" -eq 1 ] || { cat "$temp_dir/permission.out" >&2; exit 1; }
grep -q "runtime disposition drifted" "$temp_dir/permission.out"

A23I_IN="$contract" A23I_OUT="$temp_dir/authority.json" bun -e '
const value = await Bun.file(process.env.A23I_IN).json();
value.authority.sha256 = "0000000000000000000000000000000000000000000000000000000000000000";
await Bun.write(process.env.A23I_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/authority.json" >"$temp_dir/authority.out" 2>&1
authority_status=$?
set -e
[ "$authority_status" -eq 1 ] || { cat "$temp_dir/authority.out" >&2; exit 1; }
grep -q "authority digest drifted" "$temp_dir/authority.out"

A23I_IN="$contract" A23I_OUT="$temp_dir/anchor.json" bun -e '
const value = await Bun.file(process.env.A23I_IN).json();
value.runtimeEvidence[0].anchors[0] = "missing build router anchor";
await Bun.write(process.env.A23I_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/anchor.json" >"$temp_dir/anchor.out" 2>&1
anchor_status=$?
set -e
[ "$anchor_status" -eq 1 ] || { cat "$temp_dir/anchor.out" >&2; exit 1; }
grep -q "missing services/identity/src/lib.rs anchor" "$temp_dir/anchor.out"

route_runtime="$temp_dir/route-runtime"
cp -R "$runtime_root" "$route_runtime"
A23I_SOURCE="$route_runtime/services/identity/src/lib.rs" bun -e '
const path = process.env.A23I_SOURCE;
const source = await Bun.file(path).text();
const anchor = ".route(\"/health\", get(health))";
if (!source.includes(anchor)) process.exit(1);
const mutated = source.replace(anchor, `${anchor}\n        .route(\"/unapproved-anonymous\", get(health))`);
if (!mutated.includes("pub fn build_router(") || !mutated.includes("AccessPolicy::UnsafeLifecycle | AccessPolicy::Blocked")) process.exit(1);
await Bun.write(path, mutated);
'
set +e
"$verify" --mode integrity --runtime-root "$route_runtime" >"$temp_dir/route-runtime.out" 2>&1
route_runtime_status=$?
set -e
[ "$route_runtime_status" -eq 1 ] || { cat "$temp_dir/route-runtime.out" >&2; exit 1; }
grep -q "services/identity/src/lib.rs production topology byte length drifted" "$temp_dir/route-runtime.out"

public_runtime="$temp_dir/public-runtime"
cp -R "$runtime_root" "$public_runtime"
A23I_SOURCE="$public_runtime/services/identity/src/lib.rs" bun -e '
const path = process.env.A23I_SOURCE;
const source = await Bun.file(path).text();
const anchor = `(&Method::GET | &Method::HEAD, "/health") => AccessPolicy::Public,`;
if (!source.includes(anchor)) process.exit(1);
const mutated = source.replace(anchor, `${anchor}\n        (&Method::GET, "/unapproved-public") => AccessPolicy::Public,`);
if (!mutated.includes("pub fn build_router(") || !mutated.includes("AccessPolicy::UnsafeLifecycle | AccessPolicy::Blocked")) process.exit(1);
await Bun.write(path, mutated);
'
set +e
"$verify" --mode integrity --runtime-root "$public_runtime" >"$temp_dir/public-runtime.out" 2>&1
public_runtime_status=$?
set -e
[ "$public_runtime_status" -eq 1 ] || { cat "$temp_dir/public-runtime.out" >&2; exit 1; }
grep -q "services/identity/src/lib.rs production topology byte length drifted" "$temp_dir/public-runtime.out"

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

echo "identity fail-closed self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, readiness/permission/authority/anchor/extra-route/extra-public/prod/live tamper=1)"
