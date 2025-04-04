use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{authorization::Bearer, Authorization},
};
use chrono::Utc;
use firebase_rs::Firebase;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::OnceCell;

static FIREBASE_AUTH: OnceCell<Arc<FirebaseAuth>> = OnceCell::const_new();

pub struct FirebaseAuth {
    #[allow(dead_code)]
    client: Firebase,
    public_keys: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub exp: i64,
}

impl FirebaseAuth {
    pub async fn new() -> Self {
        let firebase = Firebase::new("https://YOUR_PROJECT.firebaseio.com")
            .expect("Invalid Firebase URL");

        // In a real implementation, you would fetch and cache Google's public keys
        // from https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com
        let public_keys = String::from("MOCK_KEY_FOR_DEVELOPMENT");

        Self { 
            client: firebase,
            public_keys,
        }
    }

    pub async fn initialize() -> Arc<Self> {
        FIREBASE_AUTH
            .get_or_init(|| async { Arc::new(Self::new().await) })
            .await
            .clone()
    }

    pub async fn verify_token(&self, token: &str) -> Result<Claims, StatusCode> {
        // In a real implementation, you would:
        // 1. Parse the token header to get the key ID (kid)
        // 2. Get the corresponding public key from self.public_keys
        // 3. Use that key to verify the token
        let validation = Validation::new(Algorithm::RS256);
        let key = DecodingKey::from_secret(self.public_keys.as_bytes());

        let token_data = decode::<Claims>(
            token,
            &key,
            &validation
        ).map_err(|_| StatusCode::UNAUTHORIZED)?;

        let claims = token_data.claims;

        // Check token expiration
        if claims.exp < Utc::now().timestamp() {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(claims)
    }
}

pub async fn auth_middleware(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let firebase_auth = FirebaseAuth::initialize().await;
    let token = bearer.token();
    
    let claims = firebase_auth.verify_token(token).await?;
    
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, Router};
    use axum::http::{Request, StatusCode};
    use axum::middleware::from_fn;
    use axum::routing::get;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "Protected Data"
    }

    #[tokio::test]
    async fn test_auth_middleware_unauthorized() {
        let app = Router::new()
            .route("/", get(test_handler))
            .layer(from_fn(auth_middleware));

        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
