//! Shared content server fns — `A7 B1` public + `A8 B4` media/news.
//! Unified fetch for `bff-frontend` :3000 + `bff-admin` :3001 (easy maintenance).

use dioxus::prelude::{server, ServerFnError};

#[cfg(feature = "server")]
use super::{api_base, fetch_value};

#[server]
pub async fn get_news_list_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/v1/content/news?page={}&limit={}",
        api_base(),
        page,
        limit
    ))
    .await
}

#[server]
pub async fn get_news_detail_shared(slug: String) -> Result<serde_json::Value, ServerFnError> {
    let s = slug.trim().to_string();
    if s.is_empty() || s.len() > 128 {
        return Err(ServerFnError::new("invalid slug".to_string()));
    }
    fetch_value(format!("{}/api/v1/content/news/{}", api_base(), s)).await
}

#[server]
pub async fn get_home_news_shared() -> Result<serde_json::Value, ServerFnError> {
    // Home preview is first page, small limit — reuses content list.
    fetch_value(format!("{}/api/v1/content/news?page=1&limit=3", api_base())).await
}

#[cfg(test)]
mod tests {

    use serial_test::serial;
    use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    #[serial]
    async fn get_news_list_via_wiremock() {
        let server = MockServer::start().await;
        crate::server::set_api_base_for_test(server.uri());
        let body = serde_json::json!({"items": [], "total": 0});
        Mock::given(method("GET"))
            .and(path_regex(r"/api/v1/content/news.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
        let val = crate::server::fetch_value(format!(
            "{}/api/v1/content/news?page=1&limit=10",
            server.uri()
        ))
        .await
        .expect("wiremock news list");
        assert_eq!(val["total"], 0);
        crate::server::clear_api_base_for_test();
    }

    #[tokio::test]
    #[serial]
    async fn get_news_detail_via_wiremock() {
        let server = MockServer::start().await;
        crate::server::set_api_base_for_test(server.uri());
        let body = serde_json::json!({"slug": "live-article", "title": "Live"});
        Mock::given(method("GET"))
            .and(path_regex(r"/api/v1/content/news/live-article"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
        let val = crate::server::fetch_value(format!(
            "{}/api/v1/content/news/live-article",
            server.uri()
        ))
        .await
        .expect("wiremock news detail");
        assert_eq!(val["slug"], "live-article");
        crate::server::clear_api_base_for_test();
    }
}
