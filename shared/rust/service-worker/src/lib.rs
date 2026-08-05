//! Rust/WASM public-shell recovery worker.
//!
//! The worker caches only the explicitly public `/offline` response and never
//! stores API traffic, authenticated documents, request bodies, or credentials.

pub const GENERATED_MODULE: &str = "epsx_service_worker_bootstrap.js";

#[cfg(target_arch = "wasm32")]
mod worker {
    use js_sys::{global, Array, Object, Promise, Reflect};
    use serde::Deserialize;
    use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
    use wasm_bindgen_futures::future_to_promise;
    use web_sys::{
        Event, ExtendableEvent, FetchEvent, NotificationEvent, PushEvent, Request, Response,
        ServiceWorkerGlobalScope, WindowClient,
    };

    const CACHE: &str = "epsx-public-recovery-v1";
    const OFFLINE_PATH: &str = "/offline";

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let scope = global().dyn_into::<ServiceWorkerGlobalScope>()?;
        bind_install(&scope)?;
        bind_activate(&scope)?;
        bind_fetch(&scope)?;
        bind_push(&scope)?;
        bind_notification_click(&scope)?;
        Ok(())
    }

    fn bind_install(scope: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
        let worker = scope.clone();
        let closure = Closure::<dyn FnMut(ExtendableEvent)>::new(move |event: ExtendableEvent| {
            let worker = worker.clone();
            let task = future_to_promise(async move {
                let cache = wasm_bindgen_futures::JsFuture::from(worker.caches()?.open(CACHE))
                    .await?
                    .dyn_into::<web_sys::Cache>()?;
                let request = Request::new_with_str(OFFLINE_PATH)?;
                let response =
                    wasm_bindgen_futures::JsFuture::from(worker.fetch_with_request(&request))
                        .await?
                        .dyn_into::<Response>()?;
                if response.ok()
                    && response.headers().get("x-epsx-offline-public")?.as_deref() == Some("1")
                {
                    wasm_bindgen_futures::JsFuture::from(
                        cache.put_with_request(&request, &response),
                    )
                    .await?;
                }
                Ok(JsValue::UNDEFINED)
            });
            let _ = event.wait_until(&task);
        });
        scope.add_event_listener_with_callback("install", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    fn bind_activate(scope: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
        let clients = scope.clients();
        let closure = Closure::<dyn FnMut(ExtendableEvent)>::new(move |event: ExtendableEvent| {
            let _ = event.wait_until(&clients.claim());
        });
        scope.add_event_listener_with_callback("activate", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    fn bind_fetch(scope: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
        let worker = scope.clone();
        let closure = Closure::<dyn FnMut(FetchEvent)>::new(move |event: FetchEvent| {
            let request = event.request();
            if request.method() != "GET" || request.mode() != web_sys::RequestMode::Navigate {
                return;
            }
            let worker = worker.clone();
            let network_request = request;
            let promise: Promise = future_to_promise(async move {
                match wasm_bindgen_futures::JsFuture::from(
                    worker.fetch_with_request(&network_request),
                )
                .await
                {
                    Ok(response) => Ok(response),
                    Err(_) => {
                        let cache =
                            wasm_bindgen_futures::JsFuture::from(worker.caches()?.open(CACHE))
                                .await?
                                .dyn_into::<web_sys::Cache>()?;
                        let offline = Request::new_with_str(OFFLINE_PATH)?;
                        let value = wasm_bindgen_futures::JsFuture::from(
                            cache.match_with_request(&offline),
                        )
                        .await?;
                        if value.is_undefined() {
                            Err(JsValue::from_str("offline shell unavailable"))
                        } else {
                            Ok(value)
                        }
                    }
                }
            });
            let _ = event.respond_with(&promise);
        });
        scope.add_event_listener_with_callback("fetch", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
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

    fn bind_push(scope: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
        let registration = scope.registration();
        let closure = Closure::<dyn FnMut(PushEvent)>::new(move |event: PushEvent| {
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
            if let Ok(promise) =
                registration.show_notification_with_options(&payload.title, &options)
            {
                let _ = event.wait_until(&promise);
            }
        });
        scope.add_event_listener_with_callback("push", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    fn bind_notification_click(scope: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
        let clients = scope.clients();
        let closure =
            Closure::<dyn FnMut(NotificationEvent)>::new(move |event: NotificationEvent| {
                let notification = event.notification();
                notification.close();
                let target = Reflect::get(&notification.data(), &JsValue::from_str("action_url"))
                    .ok()
                    .and_then(|value| value.as_string())
                    .filter(|value| safe_action_path(value));
                let Some(target) = target else { return };
                let clients = clients.clone();
                let task = future_to_promise(async move {
                    let matches = wasm_bindgen_futures::JsFuture::from(clients.match_all()).await?;
                    for client in Array::from(&matches).iter() {
                        let Ok(window) = client.dyn_into::<WindowClient>() else {
                            continue;
                        };
                        let _ =
                            wasm_bindgen_futures::JsFuture::from(window.navigate(&target)?).await;
                        let _ = wasm_bindgen_futures::JsFuture::from(window.focus()?).await;
                        return Ok(JsValue::UNDEFINED);
                    }
                    let _ =
                        wasm_bindgen_futures::JsFuture::from(clients.open_window(&target)).await?;
                    Ok(JsValue::UNDEFINED)
                });
                let _ = event.wait_until(&task);
            });
        scope.add_event_listener_with_callback(
            "notificationclick",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
        Ok(())
    }

    fn safe_action_path(value: &str) -> bool {
        value.starts_with('/')
            && !value.starts_with("//")
            && value.len() <= 2_048
            && !value.contains('\\')
            && !value.chars().any(char::is_control)
    }

    #[allow(dead_code)]
    fn _event_type_marker(_event: Event) {}
}
