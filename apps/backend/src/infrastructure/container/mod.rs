// Unified Container Module following DDD and Clean Architecture
// Single unified dependency injection container for domain-driven design

use url::{Host, Url};

fn siwe_authority_from_origin(origin: &str) -> Option<String> {
    let url = Url::parse(origin).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }

    let host = match url.host()? {
        Host::Domain(host) => host.to_owned(),
        Host::Ipv4(host) => host.to_string(),
        Host::Ipv6(host) => format!("[{host}]"),
    };
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host,
    })
}

pub(super) fn configured_siwe_domain() -> String {
    for variable in ["FRONTEND_URL", "NEXT_PUBLIC_APP_URL"] {
        if let Ok(origin) = std::env::var(variable) {
            if let Some(authority) = siwe_authority_from_origin(&origin) {
                return authority;
            }
        }
    }

    if std::env::var("NODE_ENV")
        .map(|value| value == "production")
        .unwrap_or(false)
        || std::env::var("RUST_ENV")
            .map(|value| value == "production")
            .unwrap_or(false)
    {
        "epsx.io".to_owned()
    } else {
        "localhost:3000".to_owned()
    }
}

// SIMPLE CONTAINER (Legacy stateful architecture)
pub mod simple_container;

// STATELESS SERVICE FACTORY (New serverless architecture)
pub mod stateless_service_factory;

// Exports - minimal container for compilation
pub use simple_container::{DomainContainer, SimpleContainer};

// New serverless exports
pub use stateless_service_factory::{
    HealthServices, RequestServices, ServiceFactory, StatelessConfig, StatelessHealthStatus,
    StatelessServiceFactory,
};

#[cfg(test)]
mod tests {
    use super::siwe_authority_from_origin;

    #[test]
    fn siwe_authority_preserves_local_port_and_rejects_non_origins() {
        assert_eq!(
            siwe_authority_from_origin("http://localhost:3000"),
            Some("localhost:3000".to_owned())
        );
        assert_eq!(
            siwe_authority_from_origin("https://dev.epsx.io"),
            Some("dev.epsx.io".to_owned())
        );
        assert_eq!(siwe_authority_from_origin("https://epsx.io/auth"), None);
        assert_eq!(siwe_authority_from_origin("ftp://epsx.io"), None);
    }
}
