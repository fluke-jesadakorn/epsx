#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-infrastructure-readiness.sh"
contract="$repo_root/docs/migration/contracts/infrastructure-readiness.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a13-self-test.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "1 supported image-key correction, 17 stop blockers" "$temp_dir/integrity.out"
grep -q "no cluster, secrets, deployment" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_code=$?
set -e
if [ "$readiness_code" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "infrastructure-readiness self-test: expected readiness exit 3, got $readiness_code" >&2
  exit 1
fi
grep -q "P0 ledger is 1 passed, 4 partial, 2 blocked" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"

# The staging overlay has a separate image-tag contract.  Keep this regression
# check next to the prod artifact audit because an image-key mismatch silently
# leaves the Rust admin BFF on the base `:dev` tag while every other staging
# service appears correctly transformed.
kubectl kustomize "$repo_root/infrastructure/kubernetes/overlays/staging" >"$temp_dir/staging-rendered.yaml"
[ "$(grep -c '^        image: epsx-admin:staging$' "$temp_dir/staging-rendered.yaml")" -eq 1 ] || {
  echo "infrastructure-readiness self-test: staging admin image transform is missing" >&2
  exit 1
}
[ "$(grep -c '^    name: epsx-staging$' "$temp_dir/staging-rendered.yaml")" -eq 1 ] || {
  echo "infrastructure-readiness self-test: staging namespace metadata is inconsistent" >&2
  exit 1
}
if grep -Eq '^        image: epsx-(backend|frontend|admin|analytics|notification|pay-bff|pay-svc):dev$' "$temp_dir/staging-rendered.yaml"; then
  echo "infrastructure-readiness self-test: staging overlay retains an unintended :dev application image" >&2
  exit 1
fi
[ "$(grep -c '^        image: epsx-notification:staging$' "$temp_dir/staging-rendered.yaml")" -eq 1 ] || {
  echo "infrastructure-readiness self-test: staging notification image transform is missing" >&2
  exit 1
}
[ "$(grep -c '^          value: staging$' "$temp_dir/staging-rendered.yaml")" -ge 1 ] || {
  echo "infrastructure-readiness self-test: staging notification environment is missing" >&2
  exit 1
}
kubectl kustomize "$repo_root/infrastructure/kubernetes/overlays/dev" >"$temp_dir/dev-rendered.yaml"
[ "$(grep -c '^    name: epsx-dev$' "$temp_dir/dev-rendered.yaml")" -eq 1 ] || {
  echo "infrastructure-readiness self-test: dev namespace metadata is inconsistent" >&2
  exit 1
}
[ "$(grep -c '^        image: epsx-notification:dev$' "$temp_dir/dev-rendered.yaml")" -eq 1 ] || {
  echo "infrastructure-readiness self-test: dev notification image transform is missing" >&2
  exit 1
}
if grep -q 'epsx-notification' "$temp_dir/report-one.json"; then
  echo "infrastructure-readiness self-test: production report unexpectedly contains notification resources" >&2
  exit 1
fi
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.resources.total !== 15 || report.images.occurrences !== 8 || report.images.unique !== 7 || report.images.devOccurrences !== 1 || report.images.digestOccurrences !== 0 || report.nodePorts.length !== 6 || report.blockers.length !== 18 || report.stopBlockers !== 17 || report.supportedFindings !== 1 || JSON.stringify(report.p0StatusCounts) !== JSON.stringify({ passed: 1, partial: 4, blocked: 2 }) || report.productionReady !== false || report.clusterAccess !== false || report.readinessExit !== 3) process.exit(1);
' "$temp_dir/report-one.json"

A13_CONTRACT_IN="$contract" A13_CONTRACT_OUT="$temp_dir/stale-anchor.json" bun -e '
const contract = await Bun.file(process.env.A13_CONTRACT_IN).json();
contract.evidence[0].anchors[0] = "tampered missing infrastructure anchor";
await Bun.write(process.env.A13_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-anchor.json" >"$temp_dir/stale-anchor.out" 2>&1
anchor_code=$?
set -e
if [ "$anchor_code" -ne 1 ]; then
  cat "$temp_dir/stale-anchor.out" >&2
  echo "infrastructure-readiness self-test: expected stale-anchor exit 1, got $anchor_code" >&2
  exit 1
fi
grep -q "missing evidence anchor" "$temp_dir/stale-anchor.out"

A13_CONTRACT_IN="$contract" A13_CONTRACT_OUT="$temp_dir/path-traversal.json" bun -e '
const contract = await Bun.file(process.env.A13_CONTRACT_IN).json();
contract.evidence[0].file = "../outside";
delete contract.evidence[0].sha256;
await Bun.write(process.env.A13_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/path-traversal.json" >"$temp_dir/path-traversal.out" 2>&1
path_code=$?
set -e
if [ "$path_code" -ne 1 ]; then
  cat "$temp_dir/path-traversal.out" >&2
  echo "infrastructure-readiness self-test: expected traversal exit 1, got $path_code" >&2
  exit 1
fi
grep -q "unsafe evidence path" "$temp_dir/path-traversal.out"

A13_CONTRACT_IN="$contract" A13_CONTRACT_OUT="$temp_dir/render-drift.json" bun -e '
const contract = await Bun.file(process.env.A13_CONTRACT_IN).json();
contract.renderExpected.deployments[0].images = ["epsx-admin:prod"];
await Bun.write(process.env.A13_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/render-drift.json" >"$temp_dir/render-drift.out" 2>&1
render_code=$?
set -e
if [ "$render_code" -ne 1 ]; then
  cat "$temp_dir/render-drift.out" >&2
  echo "infrastructure-readiness self-test: expected render-drift exit 1, got $render_code" >&2
  exit 1
fi
grep -q "rendered deployments drift" "$temp_dir/render-drift.out"

A13_CONTRACT_IN="$contract" A13_CONTRACT_OUT="$temp_dir/image-resolution-drift.json" bun -e '
const contract = await Bun.file(process.env.A13_CONTRACT_IN).json();
contract.imageResolution[1].overlayMatch = "registry.invalid/epsx/frontend";
await Bun.write(process.env.A13_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/image-resolution-drift.json" >"$temp_dir/image-resolution-drift.out" 2>&1
image_resolution_code=$?
set -e
if [ "$image_resolution_code" -ne 1 ]; then
  cat "$temp_dir/image-resolution-drift.out" >&2
  echo "infrastructure-readiness self-test: expected image-resolution drift exit 1, got $image_resolution_code" >&2
  exit 1
fi
grep -q "image resolution drift" "$temp_dir/image-resolution-drift.out"

echo "infrastructure-readiness self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, anchor/path/render/image-resolution tamper=1)"
