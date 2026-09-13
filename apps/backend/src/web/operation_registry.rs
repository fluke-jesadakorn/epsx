use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::web::middleware::OpenIDUserContext;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct OperationDefinition {
    pub operation_id: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
    pub tag: &'static str,
    pub required_scopes: &'static [&'static str],
    pub api_key_callable: bool,
    pub mutation: bool,
    pub idempotent: bool,
}

pub const OPERATIONS: &[OperationDefinition] = &[
    OperationDefinition {
        operation_id: "getDeveloperOverview",
        method: "GET",
        path: "/api/developer-portal/overview",
        summary: "Get the signed-in owner's developer overview",
        tag: "Developer portal",
        required_scopes: &["epsx:api:read"],
        api_key_callable: false,
        mutation: false,
        idempotent: false,
    },
    OperationDefinition {
        operation_id: "listDeveloperApiKeys",
        method: "GET",
        path: "/api/developer-portal/my-keys",
        summary: "List the signed-in owner's redacted API keys",
        tag: "Developer portal",
        required_scopes: &["epsx:api:read"],
        api_key_callable: false,
        mutation: false,
        idempotent: false,
    },
    OperationDefinition {
        operation_id: "createDeveloperApiKey",
        method: "POST",
        path: "/api/developer-portal/my-keys",
        summary: "Create an owner API key and reveal its secret once",
        tag: "Developer portal",
        required_scopes: &["epsx:api:read", "epsx:api:write"],
        api_key_callable: false,
        mutation: true,
        idempotent: true,
    },
    OperationDefinition {
        operation_id: "revokeDeveloperApiKey",
        method: "POST",
        path: "/api/developer-portal/my-keys/{id}/revoke",
        summary: "Revoke an owner API key",
        tag: "Developer portal",
        required_scopes: &["epsx:api:read", "epsx:api:write"],
        api_key_callable: false,
        mutation: true,
        idempotent: true,
    },
    OperationDefinition {
        operation_id: "getAnalyticsRankings",
        method: "GET",
        path: "/api/analytics/rankings",
        summary: "Get EPS stock rankings",
        tag: "Analytics",
        required_scopes: &["epsx:analytics:view"],
        api_key_callable: true,
        mutation: false,
        idempotent: false,
    },
    OperationDefinition {
        operation_id: "getAnalyticsFilters",
        method: "GET",
        path: "/api/analytics/filters",
        summary: "Get analytics filter options",
        tag: "Analytics",
        required_scopes: &["epsx:analytics:view"],
        api_key_callable: true,
        mutation: false,
        idempotent: false,
    },
    OperationDefinition {
        operation_id: "getAnalyticsCountries",
        method: "GET",
        path: "/api/analytics/countries",
        summary: "Get supported analytics countries",
        tag: "Analytics",
        required_scopes: &["epsx:analytics:view"],
        api_key_callable: true,
        mutation: false,
        idempotent: false,
    },
    OperationDefinition {
        operation_id: "getAnalyticsAvailableCountries",
        method: "GET",
        path: "/api/analytics/available-countries",
        summary: "Get countries with currently available ranking data",
        tag: "Analytics",
        required_scopes: &["epsx:analytics:view"],
        api_key_callable: true,
        mutation: false,
        idempotent: false,
    },
    OperationDefinition {
        operation_id: "getAnalyticsSectors",
        method: "GET",
        path: "/api/analytics/sectors",
        summary: "Get sectors for a country",
        tag: "Analytics",
        required_scopes: &["epsx:analytics:view"],
        api_key_callable: true,
        mutation: false,
        idempotent: false,
    },
];

fn segments_match(template: &str, actual: &str) -> bool {
    let template = template.trim_matches('/').split('/').collect::<Vec<_>>();
    let actual = actual.trim_matches('/').split('/').collect::<Vec<_>>();
    template.len() == actual.len()
        && template.iter().zip(actual).all(|(expected, value)| {
            expected.starts_with('{') && expected.ends_with('}') || *expected == value
        })
}

pub fn operation_for_request(method: &str, path: &str) -> Option<&'static OperationDefinition> {
    OPERATIONS
        .iter()
        .find(|operation| operation.method == method && segments_match(operation.path, path))
}

pub fn operation_by_id(operation_id: &str) -> Option<&'static OperationDefinition> {
    OPERATIONS
        .iter()
        .find(|operation| operation.operation_id == operation_id)
}

/// API keys are capability credentials, so they fail closed for every route
/// that is not explicitly marked callable in the registry. JWT behavior is
/// unchanged and remains governed by each route's existing authorization.
pub async fn operation_permission_guard(request: Request, next: Next) -> Response {
    let Some(context) = request.extensions().get::<OpenIDUserContext>() else {
        return next.run(request).await;
    };
    if context.auth_method != "api_key" {
        return next.run(request).await;
    }
    let path = request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map(|uri| uri.0.path())
        .unwrap_or_else(|| request.uri().path());
    let Some(operation) = operation_for_request(request.method().as_str(), path) else {
        return registry_denied("api_key_not_available_for_operation");
    };
    if !operation.api_key_callable {
        return registry_denied("jwt_session_required_for_operation");
    }
    if operation
        .required_scopes
        .iter()
        .all(|scope| epsx_contracts::permissions::has_permission(&context.permissions, scope))
    {
        next.run(request).await
    } else {
        registry_denied("api_key_scope_denied")
    }
}

fn registry_denied(reason: &'static str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "success": false,
            "error": {
                "code": 403,
                "message": "API key is not authorized for this operation",
                "reason": reason
            }
        })),
    )
        .into_response()
}

pub fn openapi_document() -> Value {
    let mut paths = Map::new();
    for operation in OPERATIONS {
        let path_item = paths
            .entry(operation.path.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let Value::Object(methods) = path_item else {
            continue;
        };
        let mut document = json!({
            "operationId": operation.operation_id,
            "summary": operation.summary,
            "tags": [operation.tag],
            "x-epsx-api-key-callable": operation.api_key_callable,
            "x-epsx-required-scopes": operation.required_scopes,
            "x-epsx-mutation": operation.mutation,
            "x-epsx-idempotent": operation.idempotent,
            "responses": {
                "200": {"description": "Successful response"},
                "400": {"description": "Malformed request"},
                "401": {"description": "Authentication required"},
                "403": {"description": "Insufficient permission"},
                "429": {"description": "Rate limit exceeded"}
            }
        });
        if operation.api_key_callable {
            document["security"] = json!([{"apiKeyBearer": operation.required_scopes}]);
        } else {
            document["security"] = json!([{"frontendJwt": operation.required_scopes}]);
        }
        if operation.idempotent {
            document["parameters"] = json!([{
                "in": "header",
                "name": "Idempotency-Key",
                "required": true,
                "schema": {"type": "string", "maxLength": 128}
            }]);
        }
        methods.insert(operation.method.to_ascii_lowercase(), document);
    }

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "EPSX Developer API",
            "version": "1.0.0",
            "description": "Canonical operation registry for EPSX developer integrations."
        },
        "servers": [{"url": "https://api.epsx.io"}],
        "paths": paths,
        "components": {
            "securitySchemes": {
                "apiKeyBearer": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "EPSX API key",
                    "description": "Customer API key returned once by the Developer Portal."
                },
                "frontendJwt": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT",
                    "description": "EPSX frontend session token; API keys are rejected."
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_ids_and_method_paths_are_unique() {
        let mut ids = HashSet::new();
        let mut routes = HashSet::new();
        for operation in OPERATIONS {
            assert!(ids.insert(operation.operation_id));
            assert!(routes.insert((operation.method, operation.path)));
            assert!(!operation
                .required_scopes
                .iter()
                .any(|scope| scope.starts_with("admin:")));
        }
    }

    #[test]
    fn openapi_is_generated_from_every_registry_operation() {
        let document = openapi_document();
        let encoded = document.to_string();
        for operation in OPERATIONS {
            assert!(encoded.contains(operation.operation_id));
        }
        assert!(encoded.contains("apiKeyBearer"));
        assert!(encoded.contains("x-epsx-api-key-callable"));
    }

    #[test]
    fn templated_routes_match_real_paths() {
        assert_eq!(
            operation_for_request(
                "POST",
                "/api/developer-portal/my-keys/00000000-0000-0000-0000-000000000000/revoke"
            )
            .map(|operation| operation.operation_id),
            Some("revokeDeveloperApiKey")
        );
    }

    #[tokio::test]
    async fn api_key_guard_uses_registry_scopes_and_fails_closed() {
        use axum::{body::Body, middleware::from_fn, routing::get, Router};
        use tower::ServiceExt;

        async fn ok() -> StatusCode {
            StatusCode::OK
        }

        let app = Router::new()
            .route("/api/analytics/rankings", get(ok))
            .route("/api/unregistered", get(ok))
            .layer(from_fn(operation_permission_guard));
        let request = |path: &'static str, scopes: Vec<&str>| {
            let mut request = axum::http::Request::builder()
                .uri(path)
                .body(Body::empty())
                .unwrap();
            request.extensions_mut().insert(OpenIDUserContext {
                sub: "0x123".to_string(),
                wallet_address: "0x123".to_string(),
                permissions: scopes.into_iter().map(str::to_string).collect(),
                token_audiences: None,
                api_key: Some(crate::web::middleware::ApiKeyIdentity {
                    id: uuid::Uuid::nil(),
                    effective_scopes: vec![],
                    rate_limits: crate::domain::developer_portal::EffectiveApiRateLimits {
                        per_minute: 1,
                        per_hour: 1,
                        per_day: 1,
                        burst: 1,
                    },
                }),
                auth_method: "api_key".to_string(),
                jti: "key".to_string(),
                exp: i64::MAX,
                iat: 0,
                auth_time: 0,
            });
            request
        };

        assert_eq!(
            app.clone()
                .oneshot(request(
                    "/api/analytics/rankings",
                    vec!["epsx:analytics:view"]
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.clone()
                .oneshot(request("/api/analytics/rankings", vec![]))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            app.oneshot(request("/api/unregistered", vec!["epsx:analytics:view"]))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
    }
}
