//! Rust/WASM public-shell recovery worker.
//!
//! The worker caches only the explicitly public `/offline` response and never
//! stores API traffic, authenticated documents, request bodies, or credentials.

pub const GENERATED_MODULE: &str = "epsx_service_worker_bootstrap.js";

#[cfg(target_arch = "wasm32")]
mod worker {
    use js_sys::{global, Array, Object, Promise, Reflect};
    use serde::Deserialize;
    use wasm_bindgen::{prelude::*, JsCast};
    use wasm_bindgen_futures::future_to_promise;
    use web_sys::{
        NotificationEvent, PushEvent, Request, Response, ServiceWorkerGlobalScope, WindowClient,
    };

    const CACHE: &str = "epsx-public-recovery-v1";
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
            let request = Request::new_with_str(OFFLINE_PATH)?;
            let response =
                wasm_bindgen_futures::JsFuture::from(worker.fetch_with_request(&request))
                    .await?
                    .dyn_into::<Response>()?;
            if response.ok()
                && response.headers().get("x-epsx-public-cache")?.as_deref()
                    == Some("offline-shell-v1")
            {
                wasm_bindgen_futures::JsFuture::from(cache.put_with_request(&request, &response))
                    .await?;
            }
            Ok(JsValue::UNDEFINED)
        })
    }

    #[wasm_bindgen]
    pub fn activate() -> Promise {
        future_to_promise(async move {
            let worker = global().dyn_into::<ServiceWorkerGlobalScope>()?;
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
