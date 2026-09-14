//! Rust/WASM public-shell recovery worker.
//!
//! The worker caches the explicitly public `/offline` response and its fixed
//! display styles, never API traffic, authenticated documents, request bodies, or credentials.

pub const GENERATED_MODULE: &str = "epsx_service_worker_bootstrap.v3.js";

pub const RECOVERY_STYLES: &[&str] = &["/public/dist/tailwind.css"];

pub fn recovery_document_has_styles(html: &str) -> bool {
    RECOVERY_STYLES.iter().all(|path| html.contains(path))
}

#[cfg(test)]
mod tests {
    #[test]
    fn tailwind_only_offline_document_can_install() {
        assert!(super::recovery_document_has_styles(
            r#"<link rel="stylesheet" href="/public/dist/tailwind.css">"#
        ));
        assert!(!super::recovery_document_has_styles(
            r#"<link rel="stylesheet" href="/public/enterprise.css?v=dioxus-2">"#
        ));
    }
}

#[cfg(target_arch = "wasm32")]
mod worker {
    use js_sys::{global, Array, Object, Promise, Reflect};
    use serde::Deserialize;
    use wasm_bindgen::{prelude::*, JsCast};
    use wasm_bindgen_futures::future_to_promise;
    use web_sys::{
        NotificationEvent, PushEvent, Request, Response, ServiceWorkerGlobalScope, WindowClient,
    };

    const CACHE: &str = "epsx-public-recovery-v4";
    const STYLES: &[&str] = super::RECOVERY_STYLES;
    const OFFLINE_PATH: &str = "/offline";

    /// Complete the public offline-shell installation after the generated
    /// bootstrap has synchronously captured the browser's install event.
    #[wasm_bindgen]
    pub fn install() -> Promise {
        future_to_promise(async move {
            let worker = global().dyn_into::<ServiceWorkerGlobalScope>()?;
            let cache = wasm_bindgen_futures::JsFuture::from(worker.caches()?.open(CACHE))
                .await?
                .dyn_into::<web_sys::Cache>()?;
            let request = recovery_request(OFFLINE_PATH)?;
            let response =
                wasm_bindgen_futures::JsFuture::from(worker.fetch_with_request(&request))
                    .await?
                    .dyn_into::<Response>()?;
            if !response.ok()
                || response.headers().get("x-epsx-public-cache")?.as_deref()
                    != Some("offline-shell-v1")
            {
                return Err(JsValue::from_str("public offline shell unavailable"));
            }
            let html = wasm_bindgen_futures::JsFuture::from(response.clone()?.text()?)
                .await?
                .as_string()
                .ok_or_else(|| JsValue::from_str("invalid offline HTML"))?;
            if !super::recovery_document_has_styles(&html) {
                return Err(JsValue::from_str(
                    "offline shell stylesheet version is stale",
                ));
            }
            let mut styles = Vec::new();
            for path in STYLES {
                let request = recovery_request(path)?;
                let response =
                    wasm_bindgen_futures::JsFuture::from(worker.fetch_with_request(&request))
                        .await?
                        .dyn_into::<Response>()?;
                if !response.ok()
                    || !response
                        .headers()
                        .get("content-type")?
                        .is_some_and(|value| value.starts_with("text/css"))
                {
                    return Err(JsValue::from_str("public recovery stylesheet unavailable"));
                }
                styles.push((request, response));
            }
            // Install succeeds only with a matching document and complete stylesheet set.
            // Rejection leaves the previous worker active and its cache intact.
            for (request, response) in styles {
                wasm_bindgen_futures::JsFuture::from(cache.put_with_request(&request, &response))
                    .await?;
            }
            wasm_bindgen_futures::JsFuture::from(cache.put_with_request(&request, &response))
                .await?;
            Ok(JsValue::UNDEFINED)
        })
    }

    fn recovery_request(path: &str) -> Result<Request, JsValue> {
        let options = web_sys::RequestInit::new();
        options.set_cache(web_sys::RequestCache::Reload);
        options.set_credentials(web_sys::RequestCredentials::Omit);
        Request::new_with_str_and_init(path, &options)
    }

    /// Only fixed, public recovery display styles are eligible for asset caching.
    #[wasm_bindgen]
    pub fn fetch_public_style(request: Request) -> Promise {
        future_to_promise(async move {
            let worker = global().dyn_into::<ServiceWorkerGlobalScope>()?;
            let url = web_sys::Url::new(&request.url())?;
            let path = format!("{}{}", url.pathname(), url.search());
            let location = Reflect::get(worker.as_ref(), &JsValue::from_str("location"))?;
            let origin = Reflect::get(&location, &JsValue::from_str("origin"))?
                .as_string()
                .unwrap_or_default();
            if request.method() != "GET"
                || !STYLES.contains(&path.as_str())
                || url.origin() != origin
            {
                return Err(JsValue::from_str("not a public recovery style"));
            }
            match wasm_bindgen_futures::JsFuture::from(worker.fetch_with_request(&request)).await {
                Ok(response) => Ok(response),
                Err(_) => {
                    let cache = wasm_bindgen_futures::JsFuture::from(worker.caches()?.open(CACHE))
                        .await?
                        .dyn_into::<web_sys::Cache>()?;
                    let response =
                        wasm_bindgen_futures::JsFuture::from(cache.match_with_request(&request))
                            .await?;
                    if response.is_undefined() {
                        Err(JsValue::from_str("recovery style unavailable"))
                    } else {
                        Ok(response)
                    }
                }
            }
        })
    }

    #[wasm_bindgen]
    pub fn activate() -> Promise {
        future_to_promise(async move {
            let worker = global().dyn_into::<ServiceWorkerGlobalScope>()?;
            let caches = worker.caches()?;
            let names = wasm_bindgen_futures::JsFuture::from(caches.keys()).await?;
            for name in Array::from(&names)
                .iter()
                .filter_map(|value| value.as_string())
            {
                if name.starts_with("epsx-public-recovery-") && name != CACHE {
                    wasm_bindgen_futures::JsFuture::from(caches.delete(&name)).await?;
                }
            }
            wasm_bindgen_futures::JsFuture::from(worker.clients().claim()).await?;
            Ok(JsValue::UNDEFINED)
        })
    }

    #[wasm_bindgen]
    pub fn fetch_navigation(request: Request) -> Promise {
        future_to_promise(async move {
            if request.method() != "GET" || request.mode() != web_sys::RequestMode::Navigate {
                return Err(JsValue::from_str(
                    "offline worker accepts only GET navigations",
                ));
            }
            let worker = global().dyn_into::<ServiceWorkerGlobalScope>()?;
            match wasm_bindgen_futures::JsFuture::from(worker.fetch_with_request(&request)).await {
                Ok(response) => Ok(response),
                Err(_) => {
                    // Cached Dioxus SSR must hydrate at its own route. Serving
                    // the /offline tree at /analytics would mount a different
                    // Router component tree when a cached WASM bundle starts.
                    let original = web_sys::Url::new(&request.url())?;
                    if original.pathname() != OFFLINE_PATH {
                        let return_path = format!("{}{}", original.pathname(), original.search());
                        let encoded = js_sys::encode_uri_component(&return_path)
                            .as_string()
                            .unwrap_or_default();
                        let destination =
                            format!("{}{OFFLINE_PATH}?return_url={encoded}", original.origin());
                        return Response::redirect(&destination).map(JsValue::from);
                    }
                    let cache = wasm_bindgen_futures::JsFuture::from(worker.caches()?.open(CACHE))
                        .await?
                        .dyn_into::<web_sys::Cache>()?;
                    let offline = Request::new_with_str(OFFLINE_PATH)?;
                    let value =
                        wasm_bindgen_futures::JsFuture::from(cache.match_with_request(&offline))
                            .await?;
                    if value.is_undefined() {
                        Err(JsValue::from_str("offline shell unavailable"))
                    } else {
                        Ok(value)
                    }
                }
            }
        })
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct PushPayload {
        title: String,
        #[serde(default)]
        body: String,
        data: serde_json::Value,
        action_url: Option<String>,
    }

    #[wasm_bindgen]
    pub fn push(event: PushEvent) -> Promise {
        future_to_promise(async move {
            let worker = global().dyn_into::<ServiceWorkerGlobalScope>()?;
            let payload = event
                .data()
                .and_then(|data| serde_json::from_str::<PushPayload>(&data.text()).ok())
                .filter(|payload| {
                    !payload.title.is_empty()
                        && payload.title.len() <= 160
                        && payload.body.len() <= 2_048
                        && !payload.title.chars().any(char::is_control)
                        && !payload.body.chars().any(|character| {
                            character.is_control() && !matches!(character, '\n' | '\t')
                        })
                        && (payload.data.is_null() || payload.data.is_object())
                        && payload.action_url.as_deref().is_none_or(safe_action_path)
                })
                .unwrap_or_else(|| PushPayload {
                    title: "EPSX notification".into(),
                    body: "Open EPSX to view the latest update.".into(),
                    data: serde_json::Value::Object(Default::default()),
                    action_url: None,
                });
            let options = web_sys::NotificationOptions::new();
            options.set_body(&payload.body);
            options.set_tag("epsx-notification");
            if let Some(action_url) = payload.action_url {
                let data = Object::new();
                let _ = Reflect::set(
                    &data,
                    &JsValue::from_str("action_url"),
                    &JsValue::from_str(&action_url),
                );
                options.set_data(data.as_ref());
            }
            let promise = worker
                .registration()
                .show_notification_with_options(&payload.title, &options)?;
            wasm_bindgen_futures::JsFuture::from(promise).await?;
            Ok(JsValue::UNDEFINED)
        })
    }

    #[wasm_bindgen]
    pub fn notification_click(event: NotificationEvent) -> Promise {
        future_to_promise(async move {
            let notification = event.notification();
            notification.close();
            let target = Reflect::get(&notification.data(), &JsValue::from_str("action_url"))
                .ok()
                .and_then(|value| value.as_string())
                .filter(|value| safe_action_path(value));
            let Some(target) = target else {
                return Ok(JsValue::UNDEFINED);
            };
            let worker = global().dyn_into::<ServiceWorkerGlobalScope>()?;
            let clients = worker.clients();
            let matches = wasm_bindgen_futures::JsFuture::from(clients.match_all()).await?;
            for client in Array::from(&matches).iter() {
                let Ok(window) = client.dyn_into::<WindowClient>() else {
                    continue;
                };
                let _ = wasm_bindgen_futures::JsFuture::from(window.navigate(&target)?).await;
                let _ = wasm_bindgen_futures::JsFuture::from(window.focus()?).await;
                return Ok(JsValue::UNDEFINED);
            }
            let _ = wasm_bindgen_futures::JsFuture::from(clients.open_window(&target)).await?;
            Ok(JsValue::UNDEFINED)
        })
    }

    fn safe_action_path(value: &str) -> bool {
        value.starts_with('/')
            && !value.starts_with("//")
            && value.len() <= 2_048
            && !value.contains('\\')
            && !value.chars().any(char::is_control)
    }
}
