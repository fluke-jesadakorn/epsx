#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/infrastructure-readiness.json"
mode=""

die() {
  echo "infrastructure-readiness: ERROR: $*" >&2
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

for tool in bun git mktemp; do
  command -v "$tool" >/dev/null 2>&1 || die "$tool is required"
done

temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a13-readiness.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM
rendered="$temp_dir/prod-rendered.yaml"
overlay="$repo_root/infrastructure/kubernetes/overlays/prod"

unset KUBERNETES_SERVICE_HOST KUBERNETES_SERVICE_PORT
export KUBECONFIG="$temp_dir/no-live-kubeconfig"
export NO_PROXY="127.0.0.1,localhost,::1"
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy

if command -v kubectl >/dev/null 2>&1 && kubectl kustomize --help >/dev/null 2>&1; then
  renderer="kubectl kustomize"
  kubectl kustomize "$overlay" --output "$rendered" || die "local kubectl kustomize render failed"
elif command -v kustomize >/dev/null 2>&1; then
  renderer="kustomize build"
  kustomize build "$overlay" --output "$rendered" || die "local kustomize build failed"
else
  die "neither local 'kubectl kustomize' nor 'kustomize build' is available"
fi

[ -s "$rendered" ] || die "renderer produced no manifest"

summary=$(bun -e '
import { readFileSync, realpathSync } from "node:fs";
import { isAbsolute, resolve, sep } from "node:path";

const [rootInput, contractPath, renderedPath, renderer] = process.argv.slice(1);
const root = realpathSync(rootInput);
const fail = (message) => {
  console.error(`infrastructure-readiness: ERROR: ${message}`);
  process.exit(1);
};
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  const parts = value.split("/");
  if (parts.some((part) => !part || part === "." || part === "..")) fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
};
const sha256 = (value) => new Bun.CryptoHasher("sha256").update(value).digest("hex");
const sorted = (values) => [...values].sort((a, b) => typeof a === "number" ? a - b : String(a).localeCompare(String(b)));
const same = (actual, expected, label) => {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) fail(`${label} drift: expected ${JSON.stringify(expected)}, observed ${JSON.stringify(actual)}`);
};

let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (contract.schemaVersion !== 1 || contract.contractId !== "A13.1-infrastructure-readiness") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "hermetic-prod-artifact-audit-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.clusterAccess !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
if (contract.overlay !== "infrastructure/kubernetes/overlays/prod") fail("prod overlay path changed");
same(contract.rendererPreference, ["kubectl kustomize", "kustomize build"], "renderer preference");
if (!contract.safety || Object.entries(contract.safety).filter(([key]) => key !== "readinessMeaning").some(([, value]) => value !== false)) fail("all safety mutation/access flags must remain false");

if (!Array.isArray(contract.evidence) || contract.evidence.length < 16) fail("at least 16 evidence records are required");
const evidenceIds = new Set();
for (const item of contract.evidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  let actualPath;
  try { actualPath = realpathSync(resolve(root, item.file)); }
  catch { fail(`missing evidence file ${item.file}`); }
  if (actualPath !== root && !actualPath.startsWith(`${root}${sep}`)) fail(`unsafe evidence path for ${item.id}: ${JSON.stringify(item.file)}`);
  const content = readFileSync(actualPath, "utf8");
  if (item.sha256 !== undefined) {
    if (!/^[0-9a-f]{64}$/.test(item.sha256)) fail(`${item.id}: invalid sha256`);
    const actualHash = sha256(content);
    if (actualHash !== item.sha256) fail(`${item.id}: stale evidence digest for ${item.file}`);
  }
  if (!Array.isArray(item.anchors) || item.anchors.length === 0) fail(`${item.id}: anchors are required`);
  for (const anchor of item.anchors) {
    if (typeof anchor !== "string" || anchor.length < 4) fail(`${item.id}: invalid evidence anchor`);
    if (!content.includes(anchor)) fail(`missing evidence anchor ${item.id} in ${item.file}: ${JSON.stringify(anchor)}`);
  }
}

const rendered = readFileSync(renderedPath, "utf8");
const rawDocs = rendered.split(/^---\s*$/m).map((value) => value.trim()).filter(Boolean);
const getKindName = (doc) => ({
  kind: doc.match(/^kind: (\S+)$/m)?.[1] ?? "",
  name: doc.match(/^metadata:\n(?:  .*\n)*?  name: (\S+)$/m)?.[1] ?? ""
});
const probeBlock = (doc, probe) => {
  const lines = doc.split(/\r?\n/);
  const start = lines.findIndex((line) => line.trim() === `${probe}Probe:`);
  if (start < 0) return "";
  const indent = lines[start].match(/^\s*/)[0].length;
  const out = [lines[start]];
  for (let index = start + 1; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.trim() && line.match(/^\s*/)[0].length <= indent) break;
    out.push(line);
  }
  return out.join("\n");
};
const readinessSignature = (doc) => {
  const block = probeBlock(doc, "readiness");
  if (!block) return "missing";
  if (block.includes("httpGet:")) {
    const path = block.match(/^\s*path: (\S+)$/m)?.[1];
    const port = block.match(/^\s*port: (\d+)$/m)?.[1];
    return `http:${path}:${port}`;
  }
  if (block.includes("tcpSocket:")) {
    const port = block.match(/^\s*port: (\d+)$/m)?.[1];
    return `tcp:${port}`;
  }
  return "other";
};

const resources = rawDocs.map((doc) => {
  const { kind, name } = getKindName(doc);
  return {
    kind,
    name,
    doc,
    images: [...doc.matchAll(/^\s*image: (\S+)$/gm)].map((match) => match[1]),
    nodePorts: [...doc.matchAll(/^\s*nodePort: (\d+)$/gm)].map((match) => Number(match[1])),
    ports: [...doc.matchAll(/^\s*port: (\d+)$/gm)].map((match) => Number(match[1])),
    targetPorts: [...doc.matchAll(/^\s*targetPort: (\d+)$/gm)].map((match) => Number(match[1])),
    type: doc.match(/^  type: (\S+)$/m)?.[1] ?? null,
    replicas: Number(doc.match(/^  replicas: (\d+)$/m)?.[1] ?? 0),
    liveness: (doc.match(/^\s*livenessProbe:/gm) ?? []).length,
    readiness: (doc.match(/^\s*readinessProbe:/gm) ?? []).length,
    startup: (doc.match(/^\s*startupProbe:/gm) ?? []).length,
    readinessSignature: readinessSignature(doc)
  };
});
if (resources.some((item) => !item.kind || !item.name)) fail("could not identify every rendered resource");

const expected = contract.renderExpected;
if (!expected || expected.namespace !== "epsx-prod") fail("invalid render expectation");
const counts = {};
for (const item of resources) counts[item.kind] = (counts[item.kind] ?? 0) + 1;
for (const [kind, count] of Object.entries(expected.resourceCounts)) if ((counts[kind] ?? 0) !== count) fail(`${kind} resource count drift: expected ${count}, observed ${counts[kind] ?? 0}`);
const namespace = resources.find((item) => item.kind === "Namespace")?.name;
if (namespace !== expected.namespace) fail(`namespace drift: ${namespace}`);

const actualServices = resources.filter((item) => item.kind === "Service").map((item) => ({
  name: item.name,
  type: item.type,
  ports: item.ports,
  targetPorts: item.targetPorts,
  nodePorts: item.nodePorts
})).sort((a, b) => a.name.localeCompare(b.name));
const expectedServices = [...expected.services].sort((a, b) => a.name.localeCompare(b.name));
same(actualServices, expectedServices, "rendered services");
const nodePorts = actualServices.flatMap((service) => service.nodePorts.map((nodePort) => ({ service: service.name, nodePort }))).sort((a, b) => a.nodePort - b.nodePort);
if (new Set(nodePorts.map((item) => item.nodePort)).size !== nodePorts.length) fail("prod render contains duplicate NodePorts");
const stagingPayPatch = readFileSync(resolve(root, "infrastructure/kubernetes/overlays/staging/patches/pay-services-nodeport.yaml"), "utf8");
const stagingPayNodePorts = [...stagingPayPatch.matchAll(/^\s*nodePort: (\d+)$/gm)].map((match) => Number(match[1]));
const declaredNodePortPairs = contract.environmentNodePortPairs;
if (!declaredNodePortPairs || !Array.isArray(declaredNodePortPairs.prod) || !Array.isArray(declaredNodePortPairs.staging) || !Array.isArray(declaredNodePortPairs.dev)) fail("environment NodePort pairs are required");
same([...nodePorts.filter((item) => item.service === "epsx-pay-svc" || item.service === "epsx-pay-bff").map((item) => item.nodePort)].sort((a, b) => a - b), [...declaredNodePortPairs.prod].sort((a, b) => a - b), "prod pay NodePort pair");
same([...stagingPayNodePorts].sort((a, b) => a - b), [...declaredNodePortPairs.staging].sort((a, b) => a - b), "staging pay NodePort pair");
if (declaredNodePortPairs.prod.some((port) => declaredNodePortPairs.staging.includes(port)) || declaredNodePortPairs.prod.some((port) => declaredNodePortPairs.dev.includes(port)) || declaredNodePortPairs.staging.some((port) => declaredNodePortPairs.dev.includes(port))) fail("environment pay NodePort pairs must be disjoint");

const actualDeployments = resources.filter((item) => item.kind === "Deployment").map((item) => ({
  name: item.name,
  images: item.images,
  replicas: item.replicas,
  liveness: item.liveness,
  readiness: item.readiness,
  startup: item.startup,
  readinessSignature: item.readinessSignature,
  dependencyChecks: []
})).sort((a, b) => a.name.localeCompare(b.name));
const expectedDeployments = [...expected.deployments].sort((a, b) => a.name.localeCompare(b.name));
same(actualDeployments, expectedDeployments, "rendered deployments");

const images = actualDeployments.flatMap((item) => item.images);
const imageSummary = {
  occurrences: images.length,
  unique: new Set(images).size,
  devOccurrences: images.filter((image) => /:dev(?:@|$)/.test(image)).length,
  digestOccurrences: images.filter((image) => /@sha256:[0-9a-f]{64}$/.test(image)).length,
  ifNotPresentOccurrences: (rendered.match(/^\s*imagePullPolicy: IfNotPresent$/gm) ?? []).length
};
same(imageSummary, expected.imageSummary, "image summary");

const envFromSecretRefs = [...rendered.matchAll(/^\s*-?\s*secretRef:\n\s+name: (\S+)$/gm)].map((match) => match[1]).sort();
const secretKeyRefNames = [...rendered.matchAll(/^\s+secretKeyRef:\n\s+key: \S+\n\s+name: (\S+)$/gm)].map((match) => match[1]);
const secretKeyRefCounts = {};
for (const name of secretKeyRefNames) secretKeyRefCounts[name] = (secretKeyRefCounts[name] ?? 0) + 1;
const payDoc = resources.find((item) => item.kind === "Deployment" && item.name === "epsx-pay-svc")?.doc ?? "";
const secretSummary = {
  secretResources: counts.Secret ?? 0,
  envFromSecretRefs,
  secretKeyRefCounts: Object.fromEntries(Object.entries(secretKeyRefCounts).sort(([a], [b]) => a.localeCompare(b))),
  paySecretRefs: (payDoc.match(/^\s*-?\s*secretRef:/gm) ?? []).length,
  literalPayDatabaseUrls: (payDoc.match(/postgresql:\/\/epsx:epsx@/g) ?? []).length,
  zeroEscrowValues: (payDoc.match(/name: ESCROW_CONTRACT\n\s+value: "0"/g) ?? []).length,
  webhookEnvEntries: (payDoc.match(/^\s*- name: \S*WEBHOOK\S*$/gim) ?? []).length
};
const expectedSecrets = { ...expected.secretSummary, envFromSecretRefs: sorted(expected.secretSummary.envFromSecretRefs) };
same(secretSummary, expectedSecrets, "secret summary");

const overlayKustomization = readFileSync(resolve(root, contract.overlay, "kustomization.yaml"), "utf8");
const actualImageTransforms = [...overlayKustomization.matchAll(/^  - name: (\S+)\n    newName: (\S+)\n    newTag: (\S+)$/gm)].map((match) => ({ name: match[1], newName: match[2], newTag: match[3] }));
const expectedImageTransforms = [
  { name: "epsx-backend", newName: "epsx-backend", newTag: "prod" },
  { name: "epsx-frontend", newName: "epsx-frontend", newTag: "prod" },
  { name: "epsx-admin", newName: "epsx-admin-frontend", newTag: "prod" },
  { name: "epsx-analytics", newName: "epsx-analytics", newTag: "wave12" },
  { name: "epsx-pay-svc", newName: "epsx-pay-svc", newTag: "wave49" },
  { name: "epsx-pay-bff", newName: "epsx-pay-bff", newTag: "wave49" }
];
same(actualImageTransforms, expectedImageTransforms, "prod image transforms");
if (actualImageTransforms.some((item) => item.name === "epsx-identity")) fail("identity transform must remain absent until an approved artifact is declared");

const expectedImageResolution = [
  { id: "backend", base: "epsx-backend:prod", overlayMatch: "epsx-backend", declaredResult: "epsx-backend:prod", rendered: "epsx-backend:prod", status: "exact-match-render-unchanged" },
  { id: "frontend", base: "epsx-frontend:dev", overlayMatch: "epsx-frontend", declaredResult: "epsx-frontend:prod", rendered: "epsx-frontend:prod", status: "effective-render-change" },
  { id: "admin", base: "epsx-admin:dev", overlayMatch: "epsx-admin", declaredResult: "epsx-admin-frontend:prod", rendered: "epsx-admin-frontend:prod", status: "effective-render-change" },
  { id: "analytics", base: "epsx-analytics:wave12", overlayMatch: "epsx-analytics", declaredResult: "epsx-analytics:wave12", rendered: "epsx-analytics:wave12", status: "exact-match-render-unchanged" },
  { id: "pay-service", base: "epsx-pay-svc:prod", overlayMatch: "epsx-pay-svc", declaredResult: "epsx-pay-svc:wave49", rendered: "epsx-pay-svc:wave49", status: "effective-render-change" },
  { id: "pay-bff", base: "epsx-pay-bff:prod", overlayMatch: "epsx-pay-bff", declaredResult: "epsx-pay-bff:wave49", rendered: "epsx-pay-bff:wave49", status: "effective-render-change" },
  { id: "identity", base: "epsx-identity:dev", overlayMatch: null, declaredResult: null, rendered: "epsx-identity:dev", status: "missing-prod-transform" }
];
same(contract.imageResolution, expectedImageResolution, "image resolution");
for (const record of contract.imageResolution) if (!images.includes(record.rendered)) fail(`${record.id}: rendered image evidence missing`);
if (contract.imageResolution.filter((record) => record.status === "effective-render-change").length !== 4) fail("exactly four visible image replacements are required");
if (contract.imageResolution.filter((record) => record.status === "exact-match-render-unchanged").length !== 2) fail("exactly two exact-key same-render replacements are required");
if (contract.imageResolution.filter((record) => record.status === "missing-prod-transform").length !== 1) fail("identity must remain the one missing prod transform");

if (!Array.isArray(contract.ingressMap) || contract.ingressMap.length !== 5) fail("five ingress records are required");
const payIngress = contract.ingressMap.find((item) => item.hostname === "pay.epsx.io");
if (!payIngress) fail("missing pay ingress record");
if (payIngress.cloudflareOrigin !== "localhost:4752" || payIngress.nodePort !== 30085 || payIngress.intendedNodePort !== 30085 || payIngress.service !== "epsx-pay-bff" || payIngress.status !== "bff-path-static-unproven") fail("pay ingress stop mapping drifted");
if (!Array.isArray(contract.candidateServices) || contract.candidateServices.length !== 9 || contract.candidateServices.some((item) => item.status !== "blocked")) fail("candidate service inventory drifted");
if (contract.candidateServices.filter((item) => item.productionBase.startsWith("absent")).length !== 8) fail("exactly eight candidate services must remain recorded as absent");

const expectedP0 = ["A0", "A1", "A2", "A3", "A4", "A5", "A6"];
if (!Array.isArray(contract.p0Dependencies) || contract.p0Dependencies.length !== expectedP0.length) fail("all seven P0 dependencies are required");
same(contract.p0Dependencies.map((item) => item.id), expectedP0, "P0 dependency order");
same(contract.p0Dependencies.map((item) => item.status), ["passed", "partial", "partial", "blocked", "partial", "partial", "blocked"], "P0 evidence status order");
const p0StatusCounts = contract.p0Dependencies.reduce((counts, item) => ({ ...counts, [item.status]: (counts[item.status] ?? 0) + 1 }), {});
same(p0StatusCounts, { passed: 1, partial: 4, blocked: 2 }, "P0 evidence status counts");
for (const key of ["immutableImages", "shadow", "canary", "rollback", "secrets", "readiness"]) if (typeof contract.releasePrerequisites?.[key] !== "string" || !contract.releasePrerequisites[key]) fail(`missing release prerequisite ${key}`);

if (!Array.isArray(contract.blockers) || contract.blockers.length !== 18) fail("exactly 18 readiness findings are required");
const blockerIds = new Set();
for (const blocker of contract.blockers) {
  if (!blocker || !/^I[0-9]{2}$/.test(blocker.id) || blockerIds.has(blocker.id)) fail(`invalid or duplicate blocker ${blocker?.id}`);
  blockerIds.add(blocker.id);
  if (blocker.id === "I01") {
    if (blocker.severity !== "resolved" || blocker.status !== "supported") fail("I01: exact image-key correction must remain supported");
  } else if (blocker.severity !== "stop" || blocker.status !== "blocked") fail(`${blocker.id}: stop state changed without readiness evidence`);
  if (!Array.isArray(blocker.evidenceIds) || blocker.evidenceIds.length === 0) fail(`${blocker.id}: evidence references required`);
  for (const id of blocker.evidenceIds) if (!evidenceIds.has(id)) fail(`${blocker.id}: unknown evidence id ${id}`);
  if (typeof blocker.summary !== "string" || typeof blocker.resolution !== "string" || !blocker.summary || !blocker.resolution) fail(`${blocker.id}: summary/resolution required`);
}
const stopBlockers = contract.blockers.filter((item) => item.severity === "stop" && item.status === "blocked");
if (stopBlockers.length !== 17) fail("exactly 17 unresolved stop blockers are required");
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 10) fail("execution order drifted");

const semanticResources = resources.map((item) => ({ kind: item.kind, name: item.name, images: item.images, nodePorts: item.nodePorts, ports: item.ports, targetPorts: item.targetPorts, type: item.type, replicas: item.replicas, liveness: item.liveness, readiness: item.readiness, startup: item.startup, readinessSignature: item.readinessSignature }));
const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  renderer,
  overlay: contract.overlay,
  semanticRenderSha256: sha256(JSON.stringify(semanticResources)),
  resources: { total: resources.length, ...Object.fromEntries(Object.entries(counts).sort(([a], [b]) => a.localeCompare(b))) },
  images: { ...imageSummary, values: sorted(new Set(images)) },
  nodePorts,
  probes: { liveness: actualDeployments.reduce((sum, item) => sum + item.liveness, 0), readiness: actualDeployments.reduce((sum, item) => sum + item.readiness, 0), startup: actualDeployments.reduce((sum, item) => sum + item.startup, 0), dependencyChecks: 0 },
  secrets: secretSummary,
  ingress: contract.ingressMap.map((item) => ({ hostname: item.hostname, status: item.status })),
  candidateServices: { total: contract.candidateServices.length, absent: 8, presentButUnsafe: 1 },
  p0StatusCounts,
  p0Dependencies: contract.p0Dependencies.map((item) => ({ id: item.id, status: item.status })),
  blockers: contract.blockers.map((item) => ({ id: item.id, category: item.category, status: item.status })),
  stopBlockers: stopBlockers.length,
  supportedFindings: contract.blockers.filter((item) => item.status === "supported").length,
  productionReady: false,
  clusterAccess: false,
  readinessExit: 3
};
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
' -- "$repo_root" "$contract" "$rendered" "$renderer") || exit 1

if [ "$mode" = "report" ]; then
  printf '%s\n' "$summary"
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "infrastructure-readiness: PASS — local $renderer render and pinned artifact integrity verified (1 supported image-key correction, 17 stop blockers)"
  echo "infrastructure-readiness: LIMIT — no cluster, secrets, deployment, Cloudflare/DNS mutation, shadow, canary, or rollback readiness was proven"
  exit 0
fi

echo "infrastructure-readiness: STOP — 17 stop blockers remain; P0 ledger is 1 passed, 4 partial, 2 blocked; readiness exit is reserved as 3" >&2
echo "infrastructure-readiness: LIMIT — artifact integrity is not deployment authorization" >&2
exit 3
