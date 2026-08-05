//! Rust/WASM public-shell recovery worker.
//!
//! The worker caches only the explicitly public `/offline` response and never
//! stores API traffic, authenticated documents, request bodies, or credentials.

pub const GENERATED_MODULE: &str = "epsx_service_worker.js";

#[cfg(target_arch = "wasm32")]
mod worker {
    use js_sys::{global, Promise};
    use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
    use wasm_bindgen_futures::future_to_promise;
    use web_sys::{
        Event, ExtendableEvent, FetchEvent, Request, Response, ServiceWorkerGlobalScope,
    };

    const CACHE: &str = "epsx-public-recovery-v1";
    const OFFLINE_PATH: &str = "/offline";

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let scope = global().dyn_into::<ServiceWorkerGlobalScope>()?;
        bind_install(&scope)?;
        bind_activate(&scope)?;
        bind_fetch(&scope)?;
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

    #[allow(dead_code)]
    fn _event_type_marker(_event: Event) {}
}
