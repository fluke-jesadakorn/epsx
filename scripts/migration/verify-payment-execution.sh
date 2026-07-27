#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/payment-execution.json"
mode=""

die() {
  echo "payment-execution: ERROR: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      [ "$#" -ge 2 ] || die "--mode requires integrity, readiness, or report"
      mode=$2
      shift 2
      ;;
    --contract)
      [ "$#" -ge 2 ] || die "--contract requires a local JSON file"
      contract=$2
      shift 2
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

case "$mode" in
  integrity|readiness|report) ;;
  *) die "--mode must be integrity, readiness, or report" ;;
esac

case "$contract" in
  http://*|https://*) die "contract must be a local file" ;;
esac
[ -f "$contract" ] || die "missing contract: $contract"
command -v bun >/dev/null 2>&1 || die "bun is required"
command -v git >/dev/null 2>&1 || die "git is required"

for name in DATABASE_URL PAY_DATABASE_URL PAYMENTS_DATABASE_URL SUBSCRIPTION_DATABASE_URL REDIS_URL RPC_URL CHAIN_RPC_URL; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts databases, Redis, or a chain"
done

for name in EPSX_ENV APP_ENV ENVIRONMENT NODE_ENV RUST_ENV DEPLOY_ENV; do
  eval "value=\${$name-}"
  normalized=$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')
  case "$normalized" in
    prod|production|prod-*|production-*|*-prod|*-production)
      die "$name identifies a production-looking environment"
      ;;
  esac
done

export NO_PROXY="127.0.0.1,localhost,::1"
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy

summary=$(bun -e '
import { readFileSync, realpathSync } from "node:fs";
import { isAbsolute, resolve, sep } from "node:path";

const [rootInput, contractPath] = process.argv.slice(1);
const root = realpathSync(rootInput);
const fail = (message) => {
  console.error(`payment-execution: ERROR: ${message}`);
  process.exit(1);
};
const exact = (label, expected, actual) => {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) fail(`${label} drifted`);
};
const git = (...args) => {
  const result = Bun.spawnSync(["git", ...args], { cwd: root, stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed`);
  return result.stdout.toString().trim();
};
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const segments = value.split("/");
  if (segments.some((segment) => !segment || segment === "." || segment === "..")) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
};
const anchored = (content, item, scope) => {
  if (typeof item.anchor !== "string" || item.anchor.length < 4) fail(`${scope} ${item.id}: invalid anchor`);
  if (!content.includes(item.anchor)) fail(`missing ${scope} anchor ${item.id} in ${item.file}`);
};

let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (contract.schemaVersion !== 1 || contract.contractId !== "A6.0-payment-execution") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "deterministic-audit-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
exact("top-level field inventory", [
  "authorityCrosswalk", "blockers", "contractId", "deploymentDependencies", "durableStateDependencies", "finalityRules",
  "idempotencyRules", "integrityExit", "nonProductionSurfaces", "ownershipRules", "productionReady", "prototypeSource",
  "purpose", "readinessExit", "remainingRuntimeDdlCorrection", "requiredExecutionOrder", "routeContracts", "safety",
  "schemaVersion", "source", "statusSemantics", "targetEvidence",
], Object.keys(contract).sort());
if (!contract.safety || Object.entries(contract.safety).filter(([key]) => key !== "readinessMeaning").some(([, value]) => value !== false)) fail("safety flags must remain false");

const source = contract.source;
if (!source || source.ref !== "origin/development" || !/^[0-9a-f]{40}$/.test(source.commit)) fail("invalid pinned source ref/commit");
const resolvedRef = git("rev-parse", `${source.ref}^{commit}`);
if (resolvedRef !== source.commit) fail(`stale source ref/commit: ${source.ref}=${resolvedRef}, contract=${source.commit}`);
if (!Array.isArray(source.evidence) || source.evidence.length < 8) fail("at least eight pinned source evidence records are required");
const expectedSourceEvidence = [
  { id: "src-submit", file: "shared/api/payments.ts", blob: "6f913d8618c12fc46b517b572b35fc51cf8ca329", anchor: "return this.client.post<TransactionStatusData>(\x27/api/payments/submit\x27, request);" },
  { id: "src-checkout-submit", file: "apps/frontend/components/payment/hooks/use-payment-flow.ts", blob: "19881dc5a9ddc9e1d35f16bc1e8d3863739d79ba", anchor: "const result = await submitTransactionAction({" },
  { id: "src-browser-receipt", file: "apps/frontend/components/payment/hooks/use-direct-token-transfer.ts", blob: "a6b943082e86a878382517c19395ad2116b1844f", anchor: "if (receiptData && receiptData.status !== \x27success\x27) {" },
  { id: "src-status-poll", file: "apps/frontend/components/payment/hooks/use-payment-polling.ts", blob: "fc6af19ee80808651ed9d5b03093df5da22d2cb9", anchor: "if (d.status === \x27confirmed\x27) { clearPoll(); setStep(\x27success\x27); refetchPlanAccess(); return; }" },
  { id: "src-admin-list", file: "apps/admin-frontend/app/payments/actions.ts", blob: "fc5d4f089825a1a87787324a2661f76eb7ac9375", anchor: "return await check403(await client.get(\x27/api/payments/admin/list\x27, params));" },
  { id: "src-confirm-cents", file: "apps/frontend/app/api/payments/confirm/route.ts", blob: "18dba8e8157aa9b05a72b31347c717acfb3ba05b", anchor: "amount: Math.round(amount * 100), // Convert to cents (integer)" },
  { id: "src-public-plans", file: "shared/api/plans.ts", blob: "e5970dcd3b92f54378359a2133ede11196014fa7", anchor: "GET /api/public/plans" },
  { id: "src-plan-switch", file: "shared/api/credits.ts", blob: "95bd3d3e76da8561e852d3e627ec6b8f21187948", anchor: "return this.client.post<PlanSwitchData>(\x27/api/payments/plans/switch\x27, { new_plan_id: newPlanId });" },
];

const evidenceIds = new Set();
for (const item of source.evidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.id}: invalid source blob`);
  const actualBlob = git("rev-parse", `${source.commit}:${item.file}`);
  if (actualBlob !== item.blob) fail(`${item.id}: stale source blob for ${item.file}`);
  const content = git("show", `${source.commit}:${item.file}`);
  anchored(content, item, "source");
}
exact("source semantic evidence pins", expectedSourceEvidence, source.evidence);

const expectedPrototypeSource = {
  auditedRefLabel: "development",
  refRole: "provenance-label-only",
  commit: "6fe4d5bb3e170ba0644c07979735482bcc0f17c6",
  evidence: [
    { id: "hist-prototype-db", file: "services/payment/src/main.rs", blob: "90520d761c6c13731d13481080c8bd60a2fda01d", anchor: "default_value = \"postgres://epsx:epsx@localhost:5432/epsx_payment\"" },
    { id: "hist-prototype-intents", file: "services/payment/src/main.rs", blob: "90520d761c6c13731d13481080c8bd60a2fda01d", anchor: "CREATE TABLE IF NOT EXISTS payment_intents (" },
    { id: "hist-prototype-escrows", file: "services/payment/src/main.rs", blob: "90520d761c6c13731d13481080c8bd60a2fda01d", anchor: "CREATE TABLE IF NOT EXISTS escrows (" },
    { id: "hist-prototype-routes", file: "services/payment/src/main.rs", blob: "90520d761c6c13731d13481080c8bd60a2fda01d", anchor: ".route(\"/api/v1/payment/intents\", post(create_intent).get(list_intents))" },
  ],
};
exact("historical prototype semantic pins", expectedPrototypeSource, contract.prototypeSource);
if (git("cat-file", "-t", `${contract.prototypeSource.commit}^{commit}`) !== "commit") fail("historical prototype commit is unavailable");
for (const item of contract.prototypeSource.evidence) {
  if (evidenceIds.has(item.id)) fail(`duplicate evidence id: ${item.id}`);
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  const actualBlob = git("rev-parse", `${contract.prototypeSource.commit}:${item.file}`);
  if (actualBlob !== item.blob) fail(`${item.id}: stale historical prototype blob`);
  anchored(git("show", `${contract.prototypeSource.commit}:${item.file}`), item, "historical prototype");
}

const expectedTargetPins = {
  "tgt-plans-producer-absent": ["apps/frontend/src/main.rs", "Market entitlements and subscription pricing remain backend-owned."],
  "tgt-subscription-compatibility-producers-absent": ["apps/frontend/src/main.rs", "Unowned wallet/session and subscription compatibility producers are"],
  "tgt-payment-producer-absent": ["apps/frontend/src/main.rs", "\"/api/v1/payment/not-an-authorized-intent\""],
  "tgt-payment-ui-unavailable": ["shared/rust/dioxus_ui/src/pages/payment.rs", "Checkout is not available right now"],
  "tgt-pay-bff-singular": ["apps/pay/src/main.rs", ".route(\"/api/v1/pay/intent/{id}/execute\", any(execute_pay))"],
  "tgt-pay-bff-service-execute": ["apps/pay/src/main.rs", "let path = format!(\"/api/v1/pay/intents/{}/execute\", id);"],
  "tgt-pay-bff-zero-parties": ["apps/pay/src/main.rs", ".unwrap_or_else(|| \"0x0000000000000000000000000000000000000000\".to_string());"],
  "tgt-pay-success-static": ["apps/pay/src/components/success_screen.rs", "Your payment has been confirmed on BSC. The recipient has been notified."],
  "tgt-gateway-payment-mount": ["services/gateway/src/lib.rs", ".route(\"/api/v1/payment/{*path}\", any(proxy_payment))"],
  "tgt-gateway-payment-deny-default": ["services/gateway/src/policy.rs", "All other Pay and legacy payment shapes remain deny-by-default."],
  "tgt-pay-routes": ["services/pay/src/main.rs", "\"/api/v1/pay/intents/{id}/confirm\","],
  "tgt-pay-schema-boundary": ["services/pay/src/main.rs", "verify_schema_compatibility(&db)"],
  "tgt-pay-confirm-unverified": ["services/pay/src/handlers/intents.rs", "let tx_hash = req"],
  "tgt-pay-confirm-split-write": ["services/pay/src/handlers/intents.rs", "UPDATE public.pay_intents SET status = \x27escrowed\x27, escrow_id = $1, tx_hash = $2"],
  "tgt-pay-webhook-inbox": ["services/pay/src/handlers/pay_webhooks.rs", "ON CONFLICT (event_id) DO NOTHING"],
  "tgt-pay-admin-force": ["services/pay/src/handlers/pay_admin.rs", "WHERE id=$1 AND status=\x27pending\x27"],
  "tgt-admin-empty-confirm": ["apps/admin/src/main.rs", ".post_with_ctx(&path, &serde_json::json!({}), &ctx)"],
  "tgt-subscription-schema-boundary": ["services/subscription/src/main.rs", "verify_schema_compatibility(&db)"],
  "tgt-subscription-active-insert": ["services/subscription/src/main.rs", "INSERT INTO public.subscriptions (user_id, plan_id, account_id, payment_token)"],
  "tgt-subscription-zero-vault": ["services/subscription/src/main.rs", "\"vault_address\": \"0x0000000000000000000000000000000000000000\""],
  "tgt-canonical-routes": ["apps/backend/src/web/routes/unified_router.rs", ".route(\"/submit\", post(submit_transaction_handler))"],
  "tgt-canonical-owner": ["apps/backend/src/web/payments/submit_tx_handler.rs", "let wallet_address = user_context.wallet_address.clone();"],
  "tgt-canonical-finality": ["apps/backend/src/infrastructure/blockchain/tx_monitor_service.rs", "min_confirmations: if is_mainnet { 15 } else { 3 },"],
  "tgt-canonical-replay-index": ["apps/backend/migrations/payments/20260220100000_add_unique_tx_hash_and_expiry/up.sql", "CREATE UNIQUE INDEX IF NOT EXISTS idx_payments_unique_tx_hash"],
  "tgt-pay-host-service": ["infrastructure/kubernetes/overlays/prod/patches/pay-services-nodeport.yaml", "nodePort: 30082"],
  "tgt-pay-host-tunnel": ["infrastructure/cloudflare/cloudflared-config.prod.yml", "service: http://localhost:4747"],
  "tgt-escrow-placeholder": ["infrastructure/kubernetes/base/pay/deployment.yaml", "{ name: ESCROW_CONTRACT, value: \"0\" }"],
  "tgt-canonical-schema": ["apps/backend/migrations/payments/00000000000001_consolidated_baseline_v4/up.sql", "CREATE TABLE payments ("],
  "tgt-pay-migration": ["services/pay/migrations/20260722060000_create_pay_store.sql", "CREATE TABLE IF NOT EXISTS public.pay_intents ("],
  "tgt-pay-db-default": ["services/pay/src/main.rs", "default_value = \"postgres://epsx:epsx@localhost:5432/epsx_pay\""],
  "tgt-pay-owner-helper": ["services/pay/src/lib.rs", "pub fn canonical_owner("],
  "tgt-pay-owner-read-policy": ["services/pay/src/lib.rs", "(&Method::GET, [\"intents\" | \"escrows\"]) => AccessPolicy::OwnerRead,"],
  "tgt-pay-mutation-stop": ["services/pay/src/lib.rs", "AccessPolicy::UnsafeFinancialMutation"],
  "tgt-subscription-migration": ["services/subscription/migrations/20260722010000_create_subscription_tables.sql", "CREATE TABLE IF NOT EXISTS public.subscription_plans ("],
  "tgt-subscription-db-default": ["services/subscription/src/main.rs", "default_value = \"postgres://epsx:epsx@localhost:5432/epsx_subscription\""],
  "tgt-subscription-routes": ["services/subscription/src/main.rs", ".route(\"/api/v1/subscription/plans\", post(create_plan).get(list_plans))"],
  "tgt-db-epsx-pay-manifest": ["infrastructure/kubernetes/base/pay/deployment.yaml", "value: \"postgresql://epsx:epsx@host.docker.internal:5432/epsx_pay?sslmode=disable\""],
  "tgt-db-epsx-pay-dev": ["infrastructure/docker/docker-compose.backend.yml", "PAYMENTS_DATABASE_URL: postgresql://${DB_USER:-epsx_user}:${DB_PASSWORD:-password}@postgres:5432/epsx_pay_dev"],
  "tgt-db-epsx-payments-env": ["infrastructure/kubernetes/scripts/create-secrets.sh", "--from-literal=PAYMENTS_DATABASE_URL=\"postgresql://${DB_USER}:${DB_PASSWORD}@host.docker.internal:5432/epsx_payments_${DB_SUFFIX}?sslmode=disable\""],
  "tgt-ddl-finding-002": ["apps/backend/src/bin/blockchain_monitor.rs", ".expect(\"Failed to create database pool\");"],
  "tgt-ddl-finding-003": ["apps/backend/src/bin/migrate.rs", "&format!(\"CREATE DATABASE \\\"{}\\\"\", db_name)"],
  "tgt-ddl-finding-004": ["apps/backend/src/main.rs", ".map_err(|e| format!(\"Failed to create database pool: {}\", e))?;"],
  "tgt-canonical-db-fallback": ["apps/backend/src/infrastructure/database/diesel_connection_manager.rs", "PAYMENTS_DATABASE_URL not set, using main database pool"],
  "tgt-db-epsx-payments-dev-init": ["infrastructure/docker/scripts/init-databases-dev.sh", "CREATE DATABASE epsx_payments_dev OWNER epsx_user;"],
  "tgt-db-epsx-payments-staging-init": ["infrastructure/docker/scripts/init-databases-staging.sh", "CREATE DATABASE epsx_payments_staging OWNER epsx_user;"],
  "tgt-db-epsx-payments-prod-init": ["infrastructure/docker/scripts/init-databases-prod.sh", "CREATE DATABASE epsx_payments_prod OWNER epsx_user;"],
  "tgt-db-epsx-payments-runtime-dev": ["infrastructure/docker/docker-compose.backend.yml", "PAYMENTS_DATABASE_URL: postgresql://${DB_USER:-epsx_user}:${DB_PASSWORD:-password}@postgres:5432/epsx_payments_dev"],
  "tgt-subscription-access-policy": ["services/subscription/src/lib.rs", "(&Method::GET, [\"plans\"]) => AccessPolicy::PlansRead,"],
};
if (!Array.isArray(contract.targetEvidence) || contract.targetEvidence.length !== Object.keys(expectedTargetPins).length) fail("exact target evidence inventory is required");
const targetEvidenceIds = new Set();
for (const item of contract.targetEvidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  targetEvidenceIds.add(item.id);
  if (["tgt-pay-runtime-ddl", "tgt-subscription-runtime-ddl"].includes(item.id)) fail(`stale runtime-DDL evidence returned: ${item.id}`);
  safeRelative(item.file, item.id);
  const candidate = resolve(root, item.file);
  let actual;
  try { actual = realpathSync(candidate); }
  catch { fail(`missing target evidence file ${item.file}`); }
  if (actual !== root && !actual.startsWith(`${root}${sep}`)) fail(`unsafe evidence path for ${item.id}: ${JSON.stringify(item.file)}`);
  anchored(readFileSync(actual, "utf8"), item, "target");
  const expectedPin = expectedTargetPins[item.id];
  if (!expectedPin || item.file !== expectedPin[0] || item.anchor !== expectedPin[1]) fail(`${item.id}: target semantic evidence pin drifted`);
}
if (Object.keys(expectedTargetPins).some((id) => !targetEvidenceIds.has(id))) fail("target semantic evidence inventory drifted");
for (const staleId of ["tgt-pay-runtime-ddl", "tgt-subscription-runtime-ddl"]) if (targetEvidenceIds.has(staleId)) fail(`stale runtime-DDL evidence returned: ${staleId}`);
const schemaBoundaryEvidence = [
  { id: "tgt-pay-schema-boundary", file: "services/pay/src/main.rs", anchor: "verify_schema_compatibility(&db)" },
  { id: "tgt-subscription-schema-boundary", file: "services/subscription/src/main.rs", anchor: "verify_schema_compatibility(&db)" },
];
for (const expected of schemaBoundaryEvidence) {
  const actual = contract.targetEvidence.find((item) => item.id === expected.id);
  if (!actual || actual.file !== expected.file || actual.anchor !== expected.anchor) fail(`${expected.id}: schema-boundary evidence drifted`);
}

const expectedAuthorityCrosswalk = {
  decision: "unresolved-do-not-cut-over-or-dual-write",
  productionWriteAuthority: null,
  systems: [
    {
      id: "canonical-backend",
      provenance: "origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db-and-current-target",
      databaseNames: ["epsx_pay_dev", "epsx_payments_dev", "epsx_payments_staging", "epsx_payments_prod", "primary-DATABASE_URL-fallback"],
      schemaTables: ["payments", "subscriptions", "stock_ranking_assignments", "payment_contexts", "wallet_credits", "credit_transactions", "payment_audit_log"],
      routePrefixes: ["/api/payments"],
      writeReachability: "existing-production-shaped-writer-not-declared-future-authority",
      evidenceIds: ["tgt-canonical-schema", "tgt-canonical-routes", "tgt-canonical-owner", "tgt-canonical-finality", "tgt-canonical-replay-index", "tgt-canonical-db-fallback", "tgt-db-epsx-pay-dev", "tgt-db-epsx-payments-runtime-dev", "tgt-db-epsx-payments-env", "tgt-db-epsx-payments-dev-init", "tgt-db-epsx-payments-staging-init", "tgt-db-epsx-payments-prod-init"],
    },
    {
      id: "development-payment-prototype",
      provenance: "development@6fe4d5bb3e170ba0644c07979735482bcc0f17c6-historical-only",
      databaseNames: ["epsx_payment"],
      schemaTables: ["payment_intents", "escrows"],
      routePrefixes: ["/api/v1/payment"],
      writeReachability: "historically-reachable-unauthenticated-runtime-ddl-and-db-only-writes",
      evidenceIds: ["hist-prototype-db", "hist-prototype-intents", "hist-prototype-escrows", "hist-prototype-routes"],
    },
    {
      id: "current-pay-candidate",
      provenance: "working-tree-target",
      databaseNames: ["epsx_pay"],
      schemaTables: ["pay_intents", "escrows", "pay_links", "pay_webhook_events"],
      routePrefixes: ["/api/v1/pay", "/api/v1/admin/pay"],
      writeReachability: "principal-or-admin-scoped-reads-reachable;owner-and-internal-mutations-404;admin-mutations-401-or-403-unless-authorized-manage-then-handler-unavailable-404",
      evidenceIds: ["tgt-pay-db-default", "tgt-pay-migration", "tgt-pay-routes", "tgt-pay-owner-helper", "tgt-pay-owner-read-policy", "tgt-pay-mutation-stop"],
    },
    {
      id: "current-subscription-candidate",
      provenance: "working-tree-target",
      databaseNames: ["epsx_subscription"],
      schemaTables: ["subscription_plans", "subscriptions"],
      routePrefixes: ["/api/v1/subscription"],
      writeReachability: "protected-plan-read-and-manage-auth-gated;owner-lifecycle-and-vault-404",
      evidenceIds: ["tgt-subscription-db-default", "tgt-subscription-migration", "tgt-subscription-routes", "tgt-subscription-access-policy", "tgt-subscription-schema-boundary"],
    },
  ],
  databaseNameCrosswalk: [
    { name: "epsx_payment", role: "historical-development-prototype-default", evidenceIds: ["hist-prototype-db"] },
    { name: "epsx_pay", role: "current-pay-default-and-kubernetes-manifest-candidate", evidenceIds: ["tgt-pay-db-default", "tgt-db-epsx-pay-manifest"] },
    { name: "epsx_pay_dev", role: "canonical-backend-compose-migrator-payments-url-candidate", evidenceIds: ["tgt-db-epsx-pay-dev"] },
    { name: "epsx_payments_dev", role: "canonical-backend-compose-runtime-payments-url-and-dev-provisioning-candidate", evidenceIds: ["tgt-db-epsx-payments-runtime-dev", "tgt-db-epsx-payments-env", "tgt-db-epsx-payments-dev-init"] },
    { name: "epsx_payments_staging|epsx_payments_prod", role: "canonical-environment-payments-databases-provisioned-through-db-suffix", evidenceIds: ["tgt-db-epsx-payments-env", "tgt-db-epsx-payments-staging-init", "tgt-db-epsx-payments-prod-init"] },
    { name: "epsx_subscription", role: "current-subscription-default-candidate", evidenceIds: ["tgt-subscription-db-default"] },
  ],
  nonEquivalenceRules: [
    "renaming epsx_payment to epsx_pay does not rename payment_intents to pay_intents",
    "the guarded A3.13 fresh-schema migration contains no ALTER TABLE, data backfill, or adoption ledger",
    "the historical escrows relation is not proven compatible with the exact current candidate relation",
    "the subscription candidate is not the canonical subscriptions or plan-access model",
  ],
};
exact("payment authority crosswalk", expectedAuthorityCrosswalk, contract.authorityCrosswalk);
for (const system of contract.authorityCrosswalk.systems) for (const id of system.evidenceIds) if (!evidenceIds.has(id)) fail(`${system.id}: unknown crosswalk evidence ${id}`);
for (const row of contract.authorityCrosswalk.databaseNameCrosswalk) for (const id of row.evidenceIds) if (!evidenceIds.has(id)) fail(`${row.name}: unknown database crosswalk evidence ${id}`);

const requiredSemanticAnchors = {
  "tgt-canonical-schema": [
    "CREATE TABLE payments (", "CREATE TABLE subscriptions (", "CREATE TABLE stock_ranking_assignments (", "CREATE TABLE payment_contexts (",
    "CREATE TABLE wallet_credits (", "CREATE TABLE credit_transactions (", "CREATE TABLE payment_audit_log (",
  ],
  "tgt-canonical-routes": [
    ".nest(\"/payments\", payment_routes)", ".route(\"/validate\", post(validate_payment_handler))", ".route(\"/submit\", post(submit_transaction_handler))",
    ".route(\"/status/{tx_hash}\", get(get_transaction_status_handler))", ".route(\"/history\", get(get_user_payment_history))", ".route(\"/admin/list\", get(admin_list_payments_handler))",
  ],
  "tgt-pay-migration": [
    "CREATE TABLE IF NOT EXISTS public.pay_intents (", "CREATE TABLE IF NOT EXISTS public.escrows (", "CREATE TABLE IF NOT EXISTS public.pay_links (", "CREATE TABLE IF NOT EXISTS public.pay_webhook_events (",
  ],
  "tgt-pay-routes": [
    "\"/api/v1/pay/intents\"", "\"/api/v1/pay/escrows\"", "\"/api/v1/pay/history/{address}\"", "\"/api/v1/admin/pay/intents\"",
  ],
  "tgt-pay-mutation-stop": [
    "AccessPolicy::UnsafePaymentsManage => {", "!principal.has_permission(PAYMENTS_MANAGE_PERMISSION)",
    "return StatusCode::NOT_FOUND.into_response();", "AccessPolicy::UnsafeFinancialMutation", "AccessPolicy::InternalIdentityUnavailable | AccessPolicy::Blocked",
  ],
  "tgt-subscription-migration": [
    "CREATE TABLE IF NOT EXISTS public.subscription_plans (", "CREATE TABLE IF NOT EXISTS public.subscriptions (",
  ],
  "tgt-subscription-routes": [
    "\"/api/v1/subscription/plans\"", "\"/api/v1/subscription/subscriptions\"", "\"/api/v1/subscription/vault/{chain_id}\"", "let app = protect_router(app, verifier);",
  ],
  "tgt-subscription-access-policy": [
    "(&Method::GET, [\"plans\"]) => AccessPolicy::PlansRead,", "(&Method::POST, [\"plans\"]) => AccessPolicy::PlansManage,",
    "AccessPolicy::OwnerIdentityUnavailable", "AccessPolicy::UnsafeVaultConfig", "!principal.has_permission(PLANS_READ_PERMISSION)",
    "!principal.has_permission(PLANS_MANAGE_PERMISSION)", "| AccessPolicy::Blocked => return StatusCode::NOT_FOUND.into_response(),",
  ],
};
for (const [id, anchors] of Object.entries(requiredSemanticAnchors)) {
  const item = contract.targetEvidence.find((candidate) => candidate.id === id);
  if (!item) fail(`${id}: semantic evidence is missing`);
  const content = readFileSync(resolve(root, item.file), "utf8");
  for (const anchor of anchors) if (!content.includes(anchor)) fail(`${id}: required semantic anchor is missing: ${anchor}`);
}

const expectedDdlCorrection = {
  scannerContract: "A3.3-runtime-ddl-triage",
  totalFindings: 3,
  payFindings: { before: 10, after: 0 },
  findings: [
    { id: "finding.002", file: "apps/backend/src/bin/blockchain_monitor.rs", line: 84, ddlKind: "CREATE DATABASE", meaning: "lexical-match-not-schema-ddl", anchor: ".expect(\"Failed to create database pool\");", evidenceId: "tgt-ddl-finding-002" },
    { id: "finding.003", file: "apps/backend/src/bin/migrate.rs", line: 74, ddlKind: "CREATE DATABASE", meaning: "runtime-database-bootstrap", anchor: "&format!(\"CREATE DATABASE \\\"{}\\\"\", db_name)", evidenceId: "tgt-ddl-finding-003" },
    { id: "finding.004", file: "apps/backend/src/main.rs", line: 38, ddlKind: "CREATE DATABASE", meaning: "lexical-match-not-schema-ddl", anchor: ".map_err(|e| format!(\"Failed to create database pool: {}\", e))?;", evidenceId: "tgt-ddl-finding-004" },
  ],
};
exact("remaining runtime DDL correction", expectedDdlCorrection, contract.remainingRuntimeDdlCorrection);
for (const finding of contract.remainingRuntimeDdlCorrection.findings) {
  if (!evidenceIds.has(finding.evidenceId)) fail(`${finding.id}: unknown DDL correction evidence ${finding.evidenceId}`);
  const line = readFileSync(resolve(root, finding.file), "utf8").split(/\r?\n/)[finding.line - 1]?.trim();
  if (line !== finding.anchor) fail(`${finding.id}: exact line/anchor drifted`);
}

const expectedRoutes = [
  "payment-submit", "payment-status", "payment-validate", "payment-history", "plan-lifecycle",
  "admin-payment", "escrow-mutations", "on-chain-webhook", "subscription-lifecycle"
];
if (!Array.isArray(contract.routeContracts) || contract.routeContracts.length !== expectedRoutes.length) fail("nine route contracts are required");
const routeIds = new Set();
for (const route of contract.routeContracts) {
  if (!route || !expectedRoutes.includes(route.id) || routeIds.has(route.id)) fail(`invalid or duplicate route contract: ${route?.id}`);
  routeIds.add(route.id);
  if (route.status !== "blocked" || typeof route.ownerKey !== "string" || !route.ownerKey) fail(`${route.id}: route must remain blocked with an owner key`);
  if (!route.source || typeof route.source.method !== "string" || typeof route.source.path !== "string" || !route.source.path.startsWith("/")) fail(`${route.id}: invalid source method/path`);
  if (!Array.isArray(route.source.body) || !Array.isArray(route.source.successStatuses) || route.source.successStatuses.length === 0) fail(`${route.id}: body/status contract is required`);
  if (!Array.isArray(route.blockerIds) || route.blockerIds.length === 0) fail(`${route.id}: blocker references are required`);
}
if (expectedRoutes.some((id) => !routeIds.has(id))) fail("route contract inventory drifted");
const expectedOwnershipObservations = {
  "payment-status": ["frontend route absent", "pay intent read is authenticated and verified-owner scoped", "pay BFF adds /status but reads intent"],
  "payment-history": ["pay service GET /api/v1/pay/history/{address} requires the path address to match the verified owner"],
  "admin-payment": ["admin BFF uses /api/v1/payment/*", "gateway blocks payment prefix", "service admin reads require admin audience plus view permission", "admin mutations return 401/403 until valid manage permission, then handler-unavailable 404"],
  "escrow-mutations": ["owner/internal mutation routes remain 404", "unreachable handler bodies contain DB-only transitions", "unreachable bodies contain redundant escrow_id and ignored signature", "participant authorization remains unimplemented"],
  "subscription-lifecycle": ["router is protected by OIDC middleware", "plan reads require admin audience plus admin:plans:read", "plan creates require admin audience plus admin:plans:manage", "owner subscription lifecycle and vault routes remain 404", "unreachable subscription create body supplies user_id and activates on insert", "unreachable vault body returns a zero address", "no deployment found"],
};
for (const [id, observed] of Object.entries(expectedOwnershipObservations)) {
  exact(`${id} ownership observation`, observed, contract.routeContracts.find((route) => route.id === id)?.targetObserved);
}
if (contract.routeContracts.find((route) => route.id === "subscription-lifecycle")?.ownerKey !== "admin-audience-plus-plan-permission-for-plan-routes;owner-identity-unavailable-for-subscription-routes") fail("subscription-lifecycle owner key drifted");

if (!Array.isArray(contract.blockers) || contract.blockers.length !== 17) fail("exactly 17 stop blockers are required");
const blockerIds = new Set();
for (const blocker of contract.blockers) {
  if (!blocker || !/^B[0-9]{2}$/.test(blocker.id) || blockerIds.has(blocker.id)) fail(`invalid or duplicate blocker: ${blocker?.id}`);
  blockerIds.add(blocker.id);
  if (blocker.severity !== "stop" || blocker.status !== "blocked") fail(`${blocker.id}: stop blocker state changed without readiness proof`);
  if (typeof blocker.summary !== "string" || typeof blocker.resolution !== "string" || !blocker.summary || !blocker.resolution) fail(`${blocker.id}: summary/resolution required`);
  if (!Array.isArray(blocker.evidenceIds) || blocker.evidenceIds.length === 0) fail(`${blocker.id}: evidence references required`);
  for (const id of blocker.evidenceIds) if (!evidenceIds.has(id)) fail(`${blocker.id}: unknown evidence id ${id}`);
}
exact("B06 corrected ownership blocker", {
  id: "B06",
  severity: "stop",
  status: "blocked",
  category: "ownership",
  summary: "Reachable pay owner/admin reads are verified-principal scoped; owner/internal mutations remain 404, admin mutations are auth-gated and reach handler-unavailable 404 only with manage permission, and inactive handler bodies still trust caller coordinates.",
  evidenceIds: ["tgt-pay-owner-helper", "tgt-pay-owner-read-policy", "tgt-pay-mutation-stop", "tgt-pay-confirm-unverified", "tgt-canonical-owner"],
  resolution: "Keep mutations fail-closed until handlers derive owner and payment coordinates from verified principal and server-authoritative state; preserve scoped reads.",
}, contract.blockers.find((blocker) => blocker.id === "B06"));
exact("B07 corrected admin reachability blocker", {
  id: "B07",
  severity: "stop",
  status: "blocked",
  category: "admin-contract",
  summary: "Admin BFF route/body contracts drift; downstream admin mutations return 401/403 without valid manage authority and handler-unavailable 404 after authorization, while empty confirm/release bodies still cannot satisfy inactive handlers.",
  evidenceIds: ["src-admin-list", "tgt-admin-empty-confirm", "tgt-pay-admin-force"],
  resolution: "Lock typed admin DTOs and enforce admin audience plus operation permissions downstream.",
}, contract.blockers.find((blocker) => blocker.id === "B07"));
exact("B12 corrected subscription blocker", {
  id: "B12",
  severity: "stop",
  status: "blocked",
  category: "subscription",
  summary: "Subscription plan reads and creates are admin-permission scoped, while owner lifecycle and vault routes remain 404; unreachable bodies still take caller user coordinates and activate immediately without payment/finality/vault proof.",
  evidenceIds: ["tgt-subscription-access-policy", "tgt-subscription-active-insert", "tgt-subscription-zero-vault"],
  resolution: "Preserve admin plan read/manage permissions and keep owner lifecycle/vault fail-closed until verified owner identity, finalized funding, period/uniqueness constraints, and nonzero vault configuration are proven.",
}, contract.blockers.find((blocker) => blocker.id === "B12"));
for (const route of contract.routeContracts) for (const id of route.blockerIds) if (!blockerIds.has(id)) fail(`${route.id}: unknown blocker ${id}`);

for (const section of ["ownershipRules", "idempotencyRules", "finalityRules"]) {
  const rules = contract[section];
  if (!Array.isArray(rules) || rules.length === 0) fail(`${section} must not be empty`);
  const ids = new Set();
  for (const rule of rules) {
    if (!rule || typeof rule.id !== "string" || ids.has(rule.id) || rule.status !== "required-unproven" || typeof rule.rule !== "string" || !rule.rule) fail(`${section}: invalid rule ${rule?.id}`);
    ids.add(rule.id);
  }
}
if (!contract.statusSemantics || contract.statusSemantics.acceptedForMonitoring !== 202 || contract.statusSemantics.foreignOrMissing !== 404 || contract.statusSemantics.conflictOrReplayMismatch !== 409) fail("status semantics drifted");
if (!Array.isArray(contract.nonProductionSurfaces) || contract.nonProductionSurfaces.length < 7) fail("non-production surface inventory is incomplete");
for (const surface of contract.nonProductionSurfaces) {
  if (!surface || typeof surface.id !== "string" || typeof surface.reason !== "string" || !surface.reason || !Array.isArray(surface.evidenceIds) || surface.evidenceIds.length === 0) fail(`invalid non-production surface ${surface?.id}`);
  for (const id of surface.evidenceIds) if (!evidenceIds.has(id)) fail(`${surface.id}: unknown evidence id ${id}`);
}
if (!Array.isArray(contract.durableStateDependencies) || contract.durableStateDependencies.length < 7) fail("durable state dependencies are incomplete");
if (!Array.isArray(contract.deploymentDependencies) || contract.deploymentDependencies.length < 6) fail("deployment dependencies are incomplete");
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 9) fail("required execution order drifted");

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  source: { ref: source.ref, commit: source.commit, evidence: source.evidence.length },
  prototypeSource: { commit: contract.prototypeSource.commit, evidence: contract.prototypeSource.evidence.length },
  targetEvidence: contract.targetEvidence.length,
  schemaBoundaryEvidence: schemaBoundaryEvidence.map((item) => item.id),
  authorityCrosswalk: {
    decision: contract.authorityCrosswalk.decision,
    productionWriteAuthority: contract.authorityCrosswalk.productionWriteAuthority,
    systems: contract.authorityCrosswalk.systems.map((item) => item.id),
    databaseNames: contract.authorityCrosswalk.databaseNameCrosswalk.map((item) => item.name),
  },
  remainingRuntimeDdl: {
    total: contract.remainingRuntimeDdlCorrection.totalFindings,
    payAfter: contract.remainingRuntimeDdlCorrection.payFindings.after,
    findings: contract.remainingRuntimeDdlCorrection.findings.map((item) => item.id),
  },
  routeContracts: contract.routeContracts.map((item) => item.id),
  rules: {
    ownership: contract.ownershipRules.length,
    idempotency: contract.idempotencyRules.length,
    finality: contract.finalityRules.length
  },
  nonProductionSurfaces: contract.nonProductionSurfaces.map((item) => item.id),
  durableStateDependencies: contract.durableStateDependencies.length,
  deploymentDependencies: contract.deploymentDependencies.length,
  blockers: contract.blockers.map((item) => ({ id: item.id, category: item.category, status: item.status })),
  productionReady: false,
  readinessExit: 3
};
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
' -- "$repo_root" "$contract") || exit 1

if [ "$mode" = "report" ]; then
  printf '%s' "$summary"
  printf '\n'
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "payment-execution: PASS — pinned evidence and contract integrity verified (17 stop blockers)"
  echo "payment-execution: LIMIT — no database, chain, deployment, or production readiness was proven"
  exit 0
fi

echo "payment-execution: STOP — 17 stop blockers remain; readiness is intentionally reserved as exit 3" >&2
echo "payment-execution: LIMIT — integrity may pass while payment execution remains non-production" >&2
exit 3
