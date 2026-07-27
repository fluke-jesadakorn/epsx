#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
FIXTURE="$ROOT/docs/migration/contracts/service-authorization.json"

if [ "$#" -ne 0 ]; then
  echo "usage: $0" >&2
  exit 64
fi

if [ ! -f "$FIXTURE" ]; then
  echo "service authorization fixture missing: $FIXTURE" >&2
  exit 1
fi

python3 - "$ROOT" "$FIXTURE" <<'PY'
from __future__ import annotations

import copy
import json
import os
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any

root = Path(sys.argv[1]).resolve()
fixture_path = Path(sys.argv[2]).resolve()

EXPECTED_SERVICES = {
    "gateway",
    "identity",
    "wallet",
    "pay",
    "subscription",
    "content",
    "notification",
    "analytics",
    "indexer",
}
EXPECTED_CLASSIFICATIONS = {
    "public-allowlist",
    "authenticated-user",
    "owner-only",
    "permission-specific-admin-operator",
    "internal-webhook",
    "router-dispatch-only",
    "blocked-by-default",
    "unknown",
}
EXPECTED_CASES = {
    "anonymous",
    "expired",
    "wrongAudience",
    "crossOwner",
    "ordinaryUser",
    "granularAdmin",
}
EXPECTED_OUTCOMES = {"allow", "deny", "conditional", "not-applicable"}
EXPECTED_STATUSES = {"blocked", "partial", "aligned"}
EXPECTED_METHODS = {"GET", "POST", "PUT", "PATCH", "DELETE", "ANY"}
STATE_CHANGING_METHODS = {"POST", "PUT", "PATCH", "DELETE", "ANY"}
ALIGNED_PROTECTED_EXCEPTIONS: set[str] = set()
PERMISSION_RE = re.compile(r"^[a-z][a-z0-9-]*:[a-z0-9*-]+:[a-z0-9*-]+$")
ID_RE = re.compile(r"^[a-z0-9]+(?:[.-][a-z0-9]+)*$")
EXPECTED_IDENTITY_SOURCE = "services/identity/src/lib.rs"
EXPECTED_IDENTITY_ROUTER_ANCHOR = (
    "pub fn build_router(verifier: Arc<dyn AccessTokenVerifier>) -> Router"
)
EXPECTED_IDENTITY_SOURCE_ANCHORS = (
    "fn classify(method: &Method, path: &str) -> AccessPolicy",
    "AccessPolicy::UnsafeLifecycle | AccessPolicy::Blocked =>",
    "AccessPolicy::AuthenticatedCandidate =>",
    "AccessPolicy::AdminPermission(required) =>",
    "authenticate_headers(state.verifier.as_ref(), request.headers()).await",
    "principal.audience != ADMIN_AUDIENCE || !literal_permission",
    'pub const USERS_READ_PERMISSION: &str = "admin:users:read";',
    'pub const USERS_CREATE_PERMISSION: &str = "admin:users:create";',
    'pub const USERS_UPDATE_PERMISSION: &str = "admin:users:update";',
    'pub const USERS_DELETE_PERMISSION: &str = "admin:users:delete";',
)
EXPECTED_IDENTITY_ROUTES = {
    "identity.get.health": (
        "GET", "/health", "public-allowlist", "none-public", None, "aligned"
    ),
    "identity.post.auth-challenge": (
        "POST", "/api/v1/identity/auth/challenge", "public-allowlist",
        "none-public", None, "blocked",
    ),
    "identity.post.auth-siwe": (
        "POST", "/api/v1/identity/auth/siwe", "public-allowlist",
        "none-public", None, "blocked",
    ),
    "identity.post.auth-refresh": (
        "POST", "/api/v1/identity/auth/refresh", "authenticated-user",
        "monolith-rs256-jwks", None, "blocked",
    ),
    "identity.get.auth-me": (
        "GET", "/api/v1/identity/auth/me", "authenticated-user",
        "monolith-rs256-jwks", None, "blocked",
    ),
    "identity.post.auth-demo": (
        "POST", "/api/v1/identity/auth/demo", "blocked-by-default",
        "identity-explicit-fail-closed", None, "blocked",
    ),
    "identity.get.users": (
        "GET", "/api/v1/identity/users", "permission-specific-admin-operator",
        "monolith-rs256-jwks", "admin:users:read", "blocked",
    ),
    "identity.post.users": (
        "POST", "/api/v1/identity/users", "permission-specific-admin-operator",
        "monolith-rs256-jwks", "admin:users:create", "blocked",
    ),
    "identity.get.user": (
        "GET", "/api/v1/identity/users/{id}",
        "permission-specific-admin-operator", "monolith-rs256-jwks",
        "admin:users:read", "blocked",
    ),
    "identity.put.user": (
        "PUT", "/api/v1/identity/users/{id}",
        "permission-specific-admin-operator", "monolith-rs256-jwks",
        "admin:users:update", "blocked",
    ),
    "identity.delete.user": (
        "DELETE", "/api/v1/identity/users/{id}",
        "permission-specific-admin-operator", "monolith-rs256-jwks",
        "admin:users:delete", "blocked",
    ),
}


def load_json(path: Path) -> dict[str, Any]:
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as error:
        raise SystemExit(f"cannot read {path}: {error}")
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as error:
        raise SystemExit(
            f"invalid JSON at {path}:{error.lineno}:{error.colno}: {error.msg}"
        )
    if not isinstance(value, dict):
        raise SystemExit("fixture root must be a JSON object")
    return value


def safe_repo_file(relative: Any, errors: list[str], context: str) -> Path | None:
    if not isinstance(relative, str) or not relative:
        errors.append(f"{context}: evidence file must be a non-empty relative path")
        return None
    path = Path(relative)
    if path.is_absolute() or ".." in path.parts:
        errors.append(f"{context}: evidence path must stay repo-relative: {relative!r}")
        return None
    resolved = (root / path).resolve()
    try:
        common = os.path.commonpath((str(root), str(resolved)))
    except ValueError:
        errors.append(f"{context}: evidence path cannot be resolved under repository")
        return None
    if common != str(root):
        errors.append(f"{context}: evidence path escapes repository: {relative!r}")
        return None
    if not resolved.is_file():
        errors.append(f"{context}: evidence file does not exist: {relative}")
        return None
    return resolved


def extract_route_calls(text: str) -> list[tuple[str, set[str]]]:
    """Return Axum .route path/method sets while ignoring nested parentheses."""
    found: list[tuple[str, set[str]]] = []
    for match in re.finditer(r"\.route\s*\(", text):
        opening = text.find("(", match.start())
        depth = 1
        i = opening + 1
        quote: str | None = None
        escaped = False
        line_comment = False
        block_comment = 0
        while i < len(text) and depth:
            char = text[i]
            nxt = text[i + 1] if i + 1 < len(text) else ""
            if line_comment:
                if char == "\n":
                    line_comment = False
                i += 1
                continue
            if block_comment:
                if char == "/" and nxt == "*":
                    block_comment += 1
                    i += 2
                    continue
                if char == "*" and nxt == "/":
                    block_comment -= 1
                    i += 2
                    continue
                i += 1
                continue
            if quote is not None:
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == quote:
                    quote = None
                i += 1
                continue
            if char == "/" and nxt == "/":
                line_comment = True
                i += 2
                continue
            if char == "/" and nxt == "*":
                block_comment = 1
                i += 2
                continue
            if char == '"':
                quote = char
                i += 1
                continue
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
            i += 1
        if depth:
            continue
        call = text[opening + 1 : i - 1]
        path_match = re.match(r'\s*"([^"]+)"\s*,', call)
        if not path_match:
            continue
        methods = {
            method.upper()
            for method in re.findall(
                r"\b(get|post|put|patch|delete|any)\s*\(",
                call[path_match.end() :],
            )
        }
        found.append((path_match.group(1), methods))
    return found


def flatten_routes(doc: dict[str, Any]) -> list[tuple[dict[str, Any], dict[str, Any]]]:
    flattened: list[tuple[dict[str, Any], dict[str, Any]]] = []
    services = doc.get("services", [])
    if not isinstance(services, list):
        return flattened
    for service in services:
        if not isinstance(service, dict):
            continue
        routes = service.get("routes", [])
        if not isinstance(routes, list):
            continue
        for route in routes:
            if isinstance(route, dict):
                flattened.append((service, route))
    return flattened


def validate(
    doc: dict[str, Any],
    *,
    check_sources: bool,
) -> list[str]:
    errors: list[str] = []

    if doc.get("schemaVersion") != 1:
        errors.append("schemaVersion must be exactly 1")
    if doc.get("artifact") != "service-authorization-baseline":
        errors.append("artifact must be service-authorization-baseline")
    scope = doc.get("scope")
    if not isinstance(scope, dict):
        errors.append("scope must be an object")
    else:
        if scope.get("phase") != "A2.0":
            errors.append("scope.phase must be A2.0")
        if scope.get("productionReadinessClaim") is not False:
            errors.append("scope.productionReadinessClaim must be false")
        purpose = scope.get("purpose")
        if not isinstance(purpose, str) or "does not assert runtime authorization" not in purpose:
            errors.append("scope.purpose must disclaim runtime authorization")

    definitions = doc.get("classificationDefinitions")
    if not isinstance(definitions, dict) or set(definitions) != EXPECTED_CLASSIFICATIONS:
        errors.append(
            "classificationDefinitions must contain exactly: "
            + ", ".join(sorted(EXPECTED_CLASSIFICATIONS))
        )

    identity_contract = doc.get("identityContract")
    if not isinstance(identity_contract, dict):
        errors.append("identityContract must be an object")
    else:
        for key in ("issuer", "algorithm", "keyDiscovery", "audience", "permissionSource", "ownershipRule"):
            if not isinstance(identity_contract.get(key), str) or not identity_contract[key]:
                errors.append(f"identityContract.{key} must be a non-empty string")
        if identity_contract.get("algorithm") != "RS256":
            errors.append("identityContract.algorithm must be RS256")

    if set(doc.get("requiredCases", [])) != EXPECTED_CASES:
        errors.append("requiredCases does not match the six required authorization cases")
    if set(doc.get("allowedCaseOutcomes", [])) != EXPECTED_OUTCOMES:
        errors.append("allowedCaseOutcomes is incomplete")
    if set(doc.get("allowedObservedStatuses", [])) != EXPECTED_STATUSES:
        errors.append("allowedObservedStatuses is incomplete")

    profiles = doc.get("caseProfiles")
    if not isinstance(profiles, dict):
        errors.append("caseProfiles must be an object")
        profiles = {}
    for name, profile in profiles.items():
        if not isinstance(profile, dict):
            errors.append(f"caseProfiles.{name} must be an object")
            continue
        if set(profile) != EXPECTED_CASES:
            errors.append(f"caseProfiles.{name} must cover exactly the six required cases")
        for case, outcome in profile.items():
            if outcome not in EXPECTED_OUTCOMES:
                errors.append(f"caseProfiles.{name}.{case} has invalid outcome {outcome!r}")

    canonical_items = doc.get("canonicalEvidence")
    canonical_by_id: dict[str, dict[str, Any]] = {}
    if not isinstance(canonical_items, list) or not canonical_items:
        errors.append("canonicalEvidence must be a non-empty array")
        canonical_items = []
    for index, item in enumerate(canonical_items):
        context = f"canonicalEvidence[{index}]"
        if not isinstance(item, dict):
            errors.append(f"{context} must be an object")
            continue
        evidence_id = item.get("id")
        if not isinstance(evidence_id, str) or not ID_RE.fullmatch(evidence_id):
            errors.append(f"{context}.id is invalid")
            continue
        if evidence_id in canonical_by_id:
            errors.append(f"duplicate canonical evidence id: {evidence_id}")
        canonical_by_id[evidence_id] = item
        path = safe_repo_file(item.get("file"), errors, context) if check_sources else None
        anchors = item.get("anchors")
        if not isinstance(anchors, list) or not anchors or not all(
            isinstance(anchor, str) and anchor for anchor in anchors
        ):
            errors.append(f"{context}.anchors must be a non-empty string array")
        elif path is not None:
            source_text = path.read_text(encoding="utf-8")
            for anchor in anchors:
                if anchor not in source_text:
                    errors.append(
                        f"{context}: anchor not found in {item.get('file')}: {anchor!r}"
                    )

    services = doc.get("services")
    if not isinstance(services, list):
        errors.append("services must be an array")
        services = []
    service_names = [
        service.get("name")
        for service in services
        if isinstance(service, dict)
    ]
    if set(service_names) != EXPECTED_SERVICES:
        errors.append(
            "services must contain exactly: " + ", ".join(sorted(EXPECTED_SERVICES))
        )
    if len(service_names) != len(set(service_names)):
        errors.append("service names must be unique")

    route_ids: set[str] = set()
    inventory_tuples: set[tuple[str, str, str]] = set()
    actual_tuples: set[tuple[str, str, str]] = set()
    route_counts: Counter[str] = Counter()
    classification_counts: Counter[str] = Counter()
    status_counts: Counter[str] = Counter()
    mutation_count = 0
    high_risk_count = 0
    source_texts: dict[str, str] = {}

    for service_index, service in enumerate(services):
        context = f"services[{service_index}]"
        if not isinstance(service, dict):
            errors.append(f"{context} must be an object")
            continue
        name = service.get("name")
        if not isinstance(name, str):
            errors.append(f"{context}.name must be a string")
            continue
        source = service.get("source")
        source_path = safe_repo_file(source, errors, context) if check_sources else None
        source_text = ""
        if source_path is not None:
            source_text = source_path.read_text(encoding="utf-8")
            source_texts[name] = source_text
            router_anchor = service.get("routerAnchor")
            if not isinstance(router_anchor, str) or router_anchor not in source_text:
                errors.append(f"{context}.routerAnchor is missing from {source}")
            for path, methods in extract_route_calls(source_text):
                if not methods:
                    errors.append(
                        f"{source}: route {path!r} has no recognized Axum method mount"
                    )
                for method in methods:
                    actual_tuples.add((name, method, path))
        if not isinstance(service.get("currentBoundary"), str) or not service["currentBoundary"]:
            errors.append(f"{context}.currentBoundary must be a non-empty string")
        routes = service.get("routes")
        if not isinstance(routes, list) or not routes:
            errors.append(f"{context}.routes must be a non-empty array")
            continue

        if name == "identity":
            if source != EXPECTED_IDENTITY_SOURCE:
                errors.append(
                    "identity.source must resolve the direct-service router/classifier in "
                    + EXPECTED_IDENTITY_SOURCE
                )
            if service.get("routerAnchor") != EXPECTED_IDENTITY_ROUTER_ANCHOR:
                errors.append("identity.routerAnchor must pin build_router in services/identity/src/lib.rs")
            if source_text:
                for identity_anchor in EXPECTED_IDENTITY_SOURCE_ANCHORS:
                    if identity_anchor not in source_text:
                        errors.append(
                            "identity direct-service router/classifier anchor missing: "
                            + repr(identity_anchor)
                        )
            identity_by_id = {
                route.get("id"): route
                for route in routes
                if isinstance(route, dict) and isinstance(route.get("id"), str)
            }
            if set(identity_by_id) != set(EXPECTED_IDENTITY_ROUTES):
                errors.append("identity routes must preserve exactly the canonical 11 route ids")
            for identity_id, expected in EXPECTED_IDENTITY_ROUTES.items():
                identity_route = identity_by_id.get(identity_id)
                if not isinstance(identity_route, dict):
                    continue
                observed = identity_route.get("observed")
                actual = (
                    identity_route.get("method"),
                    identity_route.get("path"),
                    identity_route.get("classification"),
                    identity_route.get("identitySource"),
                    identity_route.get("requiredPermission"),
                    observed.get("status") if isinstance(observed, dict) else None,
                )
                if actual != expected:
                    errors.append(
                        f"{identity_id}: strict identity boundary drift; "
                        f"expected {expected!r}, found {actual!r}"
                    )

        for route_index, route in enumerate(routes):
            route_context = f"{name}.routes[{route_index}]"
            if not isinstance(route, dict):
                errors.append(f"{route_context} must be an object")
                continue

            route_id = route.get("id")
            if not isinstance(route_id, str) or not ID_RE.fullmatch(route_id):
                errors.append(f"{route_context}.id is invalid")
            else:
                if not route_id.startswith(name + "."):
                    errors.append(f"{route_context}.id must start with {name}.")
                if route_id in route_ids:
                    errors.append(f"duplicate route id: {route_id}")
                route_ids.add(route_id)

            method = route.get("method")
            path = route.get("path")
            if method not in EXPECTED_METHODS:
                errors.append(f"{route_context}.method is invalid: {method!r}")
            if not isinstance(path, str) or not path.startswith("/"):
                errors.append(f"{route_context}.path must begin with /")
            if method in EXPECTED_METHODS and isinstance(path, str):
                route_tuple = (name, method, path)
                if route_tuple in inventory_tuples:
                    errors.append(f"duplicate mounted method/path: {route_tuple}")
                inventory_tuples.add(route_tuple)

            anchor = route.get("routeAnchor")
            if not isinstance(anchor, str) or not anchor:
                errors.append(f"{route_context}.routeAnchor must be a non-empty string")
            elif source_text and anchor not in source_text:
                errors.append(
                    f"{route_context}.routeAnchor not found in parent source {source}: {anchor!r}"
                )

            classification = route.get("classification")
            if classification not in EXPECTED_CLASSIFICATIONS:
                errors.append(f"{route_context}.classification is invalid")
            identity_source = route.get("identitySource")
            if not isinstance(identity_source, str) or not identity_source:
                errors.append(f"{route_context}.identitySource must be non-empty")

            permission = route.get("requiredPermission")
            permission_status = route.get("permissionStatus")
            if classification == "permission-specific-admin-operator":
                if not isinstance(permission, str) or not PERMISSION_RE.fullmatch(permission):
                    errors.append(
                        f"{route_context}: permission-specific route needs an exact three-part permission"
                    )
                if permission_status not in {"canonical", "proposed"}:
                    errors.append(
                        f"{route_context}.permissionStatus must be canonical or proposed"
                    )
            else:
                if permission is not None:
                    errors.append(
                        f"{route_context}: only permission-specific routes may set requiredPermission"
                    )
                if permission_status != "not-applicable":
                    errors.append(
                        f"{route_context}.permissionStatus must be not-applicable"
                    )

            ownership = route.get("ownershipKey")
            if ownership is not None and (not isinstance(ownership, str) or not ownership):
                errors.append(f"{route_context}.ownershipKey must be null or non-empty")
            if classification == "owner-only" and not ownership:
                errors.append(f"{route_context}: owner-only route needs ownershipKey")

            if not isinstance(route.get("operationKind"), str) or not route["operationKind"]:
                errors.append(f"{route_context}.operationKind must be non-empty")
            mutation = route.get("mutation")
            if not isinstance(mutation, bool):
                errors.append(f"{route_context}.mutation must be boolean")
                mutation = False
            risk = route.get("risk")
            if mutation:
                mutation_count += 1
                if classification == "public-allowlist":
                    errors.append(f"{route_context}: domain mutations cannot be public")
                if risk != "high":
                    errors.append(f"{route_context}: every inventoried mutation must be high risk")
                else:
                    high_risk_count += 1
                if method not in STATE_CHANGING_METHODS:
                    errors.append(f"{route_context}: mutation uses unexpected method {method}")
            else:
                if risk != "none":
                    errors.append(f"{route_context}: non-mutation risk must be none")
                if method in STATE_CHANGING_METHODS:
                    reason = route.get("nonDomainMutationReason")
                    if not isinstance(reason, str) or not reason:
                        errors.append(
                            f"{route_context}: non-mutating {method} needs nonDomainMutationReason"
                        )
            if method == "GET" and mutation:
                errors.append(f"{route_context}: GET cannot be marked as mutation")

            profile_name = route.get("caseProfile")
            profile = profiles.get(profile_name)
            if not isinstance(profile, dict):
                errors.append(f"{route_context}.caseProfile references no valid profile")
            else:
                if set(profile) != EXPECTED_CASES:
                    errors.append(f"{route_context}: referenced profile lacks required cases")
                if mutation:
                    for case in ("anonymous", "expired", "wrongAudience"):
                        if profile.get(case) != "deny":
                            errors.append(
                                f"{route_context}: mutation case {case} must deny"
                            )
                    if classification == "owner-only" and profile.get("crossOwner") != "deny":
                        errors.append(f"{route_context}: cross-owner mutation must deny")
                    if classification == "permission-specific-admin-operator":
                        if profile.get("ordinaryUser") != "deny":
                            errors.append(f"{route_context}: ordinary user must be denied")
                        if profile.get("granularAdmin") != "allow":
                            errors.append(f"{route_context}: granular admin must be allowed")
                    if classification == "internal-webhook":
                        if profile.get("ordinaryUser") != "deny" or profile.get("granularAdmin") != "deny":
                            errors.append(
                                f"{route_context}: external user/admin credentials cannot satisfy internal auth"
                            )

            observed = route.get("observed")
            status = None
            if not isinstance(observed, dict):
                errors.append(f"{route_context}.observed must be an object")
            else:
                status = observed.get("status")
                if status not in EXPECTED_STATUSES:
                    errors.append(f"{route_context}.observed.status is invalid")
                if not isinstance(observed.get("summary"), str) or not observed["summary"]:
                    errors.append(f"{route_context}.observed.summary must be non-empty")
            if classification == "unknown" and status != "blocked":
                errors.append(f"{route_context}: unknown policy must be blocked")
            if (
                classification not in {"public-allowlist", "unknown"}
                and status == "aligned"
                and route_id not in ALIGNED_PROTECTED_EXCEPTIONS
            ):
                errors.append(
                    f"{route_context}: known unguarded protected route must remain blocked"
                )

            dependencies = route.get("dependencies")
            if not isinstance(dependencies, list) or not dependencies:
                errors.append(f"{route_context}.dependencies must be non-empty")
            elif not all(
                isinstance(dep, str) and re.fullmatch(r"A\d+", dep)
                for dep in dependencies
            ):
                errors.append(f"{route_context}.dependencies contains invalid plan id")

            refs = route.get("canonicalEvidence")
            if not isinstance(refs, list) or not refs:
                errors.append(f"{route_context}.canonicalEvidence must be non-empty")
            else:
                for ref in refs:
                    if ref not in canonical_by_id:
                        errors.append(
                            f"{route_context}: unknown canonical evidence ref {ref!r}"
                        )

            if isinstance(path, str) and isinstance(method, str):
                route_counts[name] += 1
                if classification in EXPECTED_CLASSIFICATIONS:
                    classification_counts[classification] += 1
                if status in EXPECTED_STATUSES:
                    status_counts[status] += 1

    if check_sources:
        missing = sorted(actual_tuples - inventory_tuples)
        extra = sorted(inventory_tuples - actual_tuples)
        for service, method, path in missing:
            errors.append(
                f"current Router mount missing from fixture: {service} {method} {path}"
            )
        for service, method, path in extra:
            errors.append(
                f"fixture route has no current Router mount: {service} {method} {path}"
            )

        gateway_text = source_texts.get("gateway", "")
        rewrites = {
            (source, target)
            for source, target in re.findall(
                r'proxy_rewrite_fn!\(\s*\w+\s*,\s*\w+\s*,\s*\w+\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)',
                gateway_text,
                flags=re.DOTALL,
            )
        }
        gateway_routes = [
            route
            for service, route in flatten_routes(doc)
            if service.get("name") == "gateway"
        ]
        fixture_rewrites: set[tuple[str, str]] = set()
        for route in gateway_routes:
            target = route.get("aliasTarget")
            if target is None:
                continue
            source_path = route.get("path")
            if not isinstance(source_path, str) or not isinstance(target, str):
                errors.append(f"{route.get('id')}: invalid aliasTarget")
                continue
            suffix = "/{*path}"
            source_base = source_path[: -len(suffix)] if source_path.endswith(suffix) else source_path
            target_base = target[: -len(suffix)] if target.endswith(suffix) else target
            fixture_rewrites.add((source_base, target_base))
            if (source_base, target_base) not in rewrites:
                errors.append(
                    f"{route.get('id')}: aliasTarget does not match proxy_rewrite_fn macro"
                )
        for source_path, target in sorted(rewrites - fixture_rewrites):
            errors.append(
                f"gateway rewrite lacks explicit aliasTarget: {source_path} -> {target}"
            )

    summary = doc.get("inventorySummary")
    if not isinstance(summary, dict):
        errors.append("inventorySummary must be an object")
    else:
        checks = {
            "serviceCount": len(services),
            "routeCount": sum(route_counts.values()),
            "mutationCount": mutation_count,
            "highRiskMutationCount": high_risk_count,
            "byService": dict(route_counts),
            "byClassification": dict(classification_counts),
            "byObservedStatus": dict(status_counts),
        }
        for key, calculated in checks.items():
            if summary.get(key) != calculated:
                errors.append(
                    f"inventorySummary.{key} is stale: expected {calculated!r}, found {summary.get(key)!r}"
                )

    if not isinstance(doc.get("evidenceResolution"), str) or not doc["evidenceResolution"]:
        errors.append("evidenceResolution must explain route source/anchor composition")

    return errors


doc = load_json(fixture_path)
problems = validate(doc, check_sources=True)
if problems:
    print("service authorization fixture integrity: FAIL", file=sys.stderr)
    for problem in problems:
        print(f"  - {problem}", file=sys.stderr)
    raise SystemExit(1)

# Minimal negative self-tests ensure the validator itself rejects common drift.
self_tests = 0

missing_route = copy.deepcopy(doc)
missing_route["services"][0]["routes"].pop()
if not validate(missing_route, check_sources=False):
    raise SystemExit("validator self-test failed: missing route was accepted")
self_tests += 1

public_mutation = copy.deepcopy(doc)
target = next(
    route
    for _, route in flatten_routes(public_mutation)
    if route.get("mutation") is True
)
target["classification"] = "public-allowlist"
target["caseProfile"] = "public"
if not any(
    "domain mutations cannot be public" in error
    for error in validate(public_mutation, check_sources=False)
):
    raise SystemExit("validator self-test failed: public mutation was accepted")
self_tests += 1

bad_negative_case = copy.deepcopy(doc)
bad_negative_case["caseProfiles"]["owner"]["crossOwner"] = "allow"
if not any(
    "cross-owner mutation must deny" in error
    for error in validate(bad_negative_case, check_sources=False)
):
    raise SystemExit("validator self-test failed: cross-owner mutation was accepted")
self_tests += 1

legacy_identity_source = copy.deepcopy(doc)
identity_service = next(
    service for service in legacy_identity_source["services"]
    if service.get("name") == "identity"
)
identity_service["source"] = "services/identity/src/main.rs"
if not any(
    "identity.source must resolve" in error
    for error in validate(legacy_identity_source, check_sources=False)
):
    raise SystemExit("validator self-test failed: legacy identity source was accepted")
self_tests += 1

enabled_lifecycle = copy.deepcopy(doc)
challenge = next(
    route for _, route in flatten_routes(enabled_lifecycle)
    if route.get("id") == "identity.post.auth-challenge"
)
challenge["observed"]["status"] = "aligned"
if not any(
    "strict identity boundary drift" in error
    for error in validate(enabled_lifecycle, check_sources=False)
):
    raise SystemExit("validator self-test failed: enabled identity lifecycle was accepted")
self_tests += 1

wildcard_identity_permission = copy.deepcopy(doc)
read_users = next(
    route for _, route in flatten_routes(wildcard_identity_permission)
    if route.get("id") == "identity.get.users"
)
read_users["requiredPermission"] = "admin:users:*"
if not any(
    "strict identity boundary drift" in error
    for error in validate(wildcard_identity_permission, check_sources=False)
):
    raise SystemExit("validator self-test failed: wildcard identity permission was accepted")
self_tests += 1

legacy_identity_anchor = copy.deepcopy(doc)
identity_service = next(
    service for service in legacy_identity_anchor["services"]
    if service.get("name") == "identity"
)
identity_service["routerAnchor"] = "let app = Router::new()"
if not any(
    "identity.routerAnchor must pin build_router" in error
    for error in validate(legacy_identity_anchor, check_sources=False)
):
    raise SystemExit("validator self-test failed: legacy identity router anchor was accepted")
self_tests += 1

summary = doc["inventorySummary"]
print("service authorization fixture integrity: PASS")
print("runtime authorization / production readiness: NOT PROVEN")
print(
    "services={serviceCount} routes={routeCount} mutations={mutationCount} "
    "high-risk-mutations={highRiskMutationCount}".format(**summary)
)
print(
    "classifications="
    + ",".join(
        f"{name}:{count}"
        for name, count in sorted(summary["byClassification"].items())
    )
)
print(
    "observed="
    + ",".join(
        f"{name}:{count}"
        for name, count in sorted(summary["byObservedStatus"].items())
    )
)
print(f"validator-negative-self-tests={self_tests}/7")
PY
