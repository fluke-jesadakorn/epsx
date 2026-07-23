#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-frontend-live-data.sh"
contract="$repo_root/docs/migration/contracts/frontend-live-data.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-frontend-live-data.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "PASS integrity (28 routes; 1 aligned, 10 partial, 17 blocked" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "frontend-live-data self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "STOP readiness (27 non-aligned routes" "$temp_dir/readiness.out"

"$verify" --mode emit >"$temp_dir/emit-one.json"
"$verify" --mode emit >"$temp_dir/emit-two.json"
cmp "$temp_dir/emit-one.json" "$temp_dir/emit-two.json"
bun -e 'const report = await Bun.file(process.argv[1]).json(); if (report.routeCount !== 28 || report.productionReady !== false || report.readinessExit !== 3 || report.statuses.aligned !== 1 || report.statuses.partial !== 10 || report.statuses.blocked !== 17) process.exit(1);' "$temp_dir/emit-one.json"
bun -e '
const contract = await Bun.file(process.argv[1]).json();
const docs = contract.routes.find(route => route.path === "/developer/docs");
if (!docs || docs.status !== "partial" || docs.loader.kind !== "version-pinned-static" || docs.loader.endpoints.length !== 0 || docs.interactions.search.length !== 0 || docs.hydration.status !== "implemented" || docs.blockers.length !== 1) process.exit(1);
const offline = contract.routes.find(route => route.path === "/offline");
if (!offline || offline.status !== "partial" || offline.loader.kind !== "public-service-worker-shell" || JSON.stringify(offline.loader.endpoints) !== JSON.stringify(["GET /service-worker.js"]) || offline.loader.evidence.length !== 2 || offline.hydration.status !== "implemented" || offline.blockers.length !== 1 || !offline.blockers[0].includes("copy drift")) process.exit(1);
const news = contract.routes.find(route => route.path === "/news");
if (!news || news.status !== "partial" || news.loader.kind !== "gateway-strict" || JSON.stringify(news.loader.endpoints) !== JSON.stringify(["GET /api/v1/content/news?page=1&limit=100"]) || news.payloads.staticOrSample.length !== 0 || news.states.empty !== "present" || news.states.error !== "present" || news.states.retry !== "present" || news.authOwner.auth !== "public" || !news.interactions.forms.some(item => item.includes("removed public limit")) || !news.interactions.pagination.some(item => item.includes("Previous page")) || !news.blockers.some(blocker => blocker.includes("at most 100"))) process.exit(1);
const newsDetail = contract.routes.find(route => route.path === "/news/:slug");
if (!newsDetail || newsDetail.status !== "partial" || newsDetail.loader.kind !== "gateway-strict" || JSON.stringify(newsDetail.loader.endpoints) !== JSON.stringify(["GET /api/v1/content/news/{slug}"]) || newsDetail.payloads.staticOrSample.length !== 0 || newsDetail.states.empty !== "present" || newsDetail.states.error !== "present" || newsDetail.states.retry !== "present" || newsDetail.authOwner.auth !== "public" || !newsDetail.blockers.some(blocker => blocker.includes("unknown slugs")) || !newsDetail.blockers.some(blocker => blocker.includes("outer metadata"))) process.exit(1);
const notifications = contract.routes.find(route => route.path === "/notifications");
if (!notifications || notifications.status !== "partial" || notifications.loader.kind !== "owner-gateway-explicit-outcome-plus-authenticated-shared-header" || JSON.stringify(notifications.loader.endpoints) !== JSON.stringify(["GET /api/v1/notification/list", "GET /api/v1/notifications/unread-count"]) || notifications.loader.evidence.length !== 6 || notifications.payloads.staticOrSample.length !== 0 || notifications.states.loading !== "missing" || notifications.states.empty !== "present" || notifications.states.error !== "present" || notifications.states.retry !== "present" || notifications.hydration.need !== "browser" || notifications.hydration.status !== "partial" || notifications.blockers.length !== 2 || !notifications.blockers[0].includes("live-service and browser runtime proof remain missing")) process.exit(1);
' "$contract"

FRONTEND_CONTRACT_IN="$contract" FRONTEND_CONTRACT_OUT="$temp_dir/tampered.json" bun -e '
const value = await Bun.file(process.env.FRONTEND_CONTRACT_IN).json();
value.routes[0].path = "/tampered";
await Bun.write(process.env.FRONTEND_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/tampered.json" >"$temp_dir/tampered.out" 2>&1
tampered_status=$?
set -e
[ "$tampered_status" -eq 1 ] || { cat "$temp_dir/tampered.out" >&2; exit 1; }
grep -q "not in the checked frontend route inventory" "$temp_dir/tampered.out"

FRONTEND_CONTRACT_IN="$contract" FRONTEND_CONTRACT_OUT="$temp_dir/traversal.json" bun -e '
const value = await Bun.file(process.env.FRONTEND_CONTRACT_IN).json();
value.routes[0].target.file = "../outside.rs";
await Bun.write(process.env.FRONTEND_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/traversal.json" >"$temp_dir/traversal.out" 2>&1
traversal_status=$?
set -e
[ "$traversal_status" -eq 1 ] || { cat "$temp_dir/traversal.out" >&2; exit 1; }
grep -q "unsafe evidence path" "$temp_dir/traversal.out"

FRONTEND_CONTRACT_IN="$contract" FRONTEND_CONTRACT_OUT="$temp_dir/stale-anchor.json" bun -e '
const value = await Bun.file(process.env.FRONTEND_CONTRACT_IN).json();
value.routes[0].target.anchors[0] = "tampered stale target anchor";
await Bun.write(process.env.FRONTEND_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-anchor.json" >"$temp_dir/stale-anchor.out" 2>&1
anchor_status=$?
set -e
[ "$anchor_status" -eq 1 ] || { cat "$temp_dir/stale-anchor.out" >&2; exit 1; }
grep -q "missing target anchor" "$temp_dir/stale-anchor.out"

FRONTEND_CONTRACT_IN="$contract" FRONTEND_CONTRACT_OUT="$temp_dir/wrong-existing-notification-anchor.json" bun -e '
const value = await Bun.file(process.env.FRONTEND_CONTRACT_IN).json();
value.routes.find(route => route.path === "/notifications").target.anchors[1] = "struct ServiceNotificationList {";
await Bun.write(process.env.FRONTEND_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/wrong-existing-notification-anchor.json" >"$temp_dir/wrong-existing-notification-anchor.out" 2>&1
wrong_notification_status=$?
set -e
[ "$wrong_notification_status" -eq 1 ] || { cat "$temp_dir/wrong-existing-notification-anchor.out" >&2; exit 1; }
grep -q "/notifications truthful read-only semantic contract drifted" "$temp_dir/wrong-existing-notification-anchor.out"

FRONTEND_CONTRACT_IN="$contract" FRONTEND_CONTRACT_OUT="$temp_dir/wrong-existing-notification-badge-anchor.json" bun -e '
const value = await Bun.file(process.env.FRONTEND_CONTRACT_IN).json();
value.routes.find(route => route.path === "/notifications").loader.evidence[4].anchor = "let outcome = crate::api::load_owner_notifications(";
await Bun.write(process.env.FRONTEND_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/wrong-existing-notification-badge-anchor.json" >"$temp_dir/wrong-existing-notification-badge-anchor.out" 2>&1
wrong_notification_badge_status=$?
set -e
[ "$wrong_notification_badge_status" -eq 1 ] || { cat "$temp_dir/wrong-existing-notification-badge-anchor.out" >&2; exit 1; }
grep -q "/notifications truthful read-only semantic contract drifted" "$temp_dir/wrong-existing-notification-badge-anchor.out"

echo "frontend-live-data self-test: PASS (integrity=0, readiness-stop=3, deterministic emit, tamper/path/stale-anchor/wrong-existing-notification+badge-anchor=1)"
