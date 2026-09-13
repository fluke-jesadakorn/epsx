//! Typed media inventory and commands; storage permissions remain upstream.
use super::{
    admin::{AdminAnalyticsShell, AdminNavigation},
    LoadError,
};
use crate::pages::admin_pages::media::*;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MediaCommand {
    Upload { filename: String, bytes: Vec<u8> },
    Delete { bucket: String, key: String },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaRequest {
    pub command: MediaCommand,
    pub idempotency_key: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MediaFailure {
    Invalid,
    Forbidden,
    Unauthenticated,
    Conflict,
    Unavailable,
    Malformed,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct MediaProvider {
    pub read: MediaProviderReadCallback,
    pub mutate: MediaProviderMutateCallback,
}
#[server(prefix = "/_server/admin", endpoint = "media_read")]
pub async fn read_media(
    bucket: String,
) -> Result<Result<AdminMediaList, LoadError>, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<MediaProvider>,
        _,
    >()
    .await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(bucket, headers).await)
}
#[server(prefix = "/_server/admin", endpoint = "media_change")]
pub async fn change_media(
    request: MediaRequest,
) -> Result<Result<AdminMediaMutationProjection, MediaFailure>, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<MediaProvider>,
        _,
    >()
    .await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.mutate)(request, headers).await)
}
#[derive(Clone, Copy)]
struct MediaEvents {
    submit: EventHandler<FormEvent>,
    choose: EventHandler<FormEvent>,
    navigate: EventHandler<String>,
}
pub fn submit(event: FormEvent) {
    if let Some(h) = try_consume_context::<MediaEvents>() {
        event.prevent_default();
        h.submit.call(event);
    }
}
pub fn choose_file(event: FormEvent) {
    if let Some(h) = try_consume_context::<MediaEvents>() {
        h.choose.call(event);
    }
}
pub fn navigate(event: MouseEvent, url: String) {
    if let Some(h) = try_consume_context::<MediaEvents>() {
        event.prevent_default();
        h.navigate.call(url);
    }
}
fn bucket_query(query: &str) -> Result<String, LoadError> {
    let mut bucket = None;
    let mut seen = std::collections::HashSet::new();
    for (name, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if !seen.insert(name.to_string()) {
            return Err(LoadError::InvalidQuery);
        }
        match name.as_ref() {
            "bucket" if matches!(value.as_ref(), "news" | "public") => {
                bucket = Some(value.into_owned())
            }
            // Preserve legacy form redirect URLs without trusting query strings
            // as proof that any storage operation actually succeeded.
            "mutation"
                if matches!(
                    value.as_ref(),
                    "committed"
                        | "conflict"
                        | "forbidden"
                        | "unauthorized"
                        | "unavailable"
                        | "malformed"
                ) => {}
            "key"
                if !value.is_empty()
                    && value.len() <= 1024
                    && value.trim() == value
                    && !value.chars().any(char::is_control) => {}
            "size" if value.parse::<u64>().is_ok() => {}
            "deleted" if matches!(value.as_ref(), "true" | "false") => {}
            _ => return Err(LoadError::InvalidQuery),
        }
    }
    Ok(bucket.unwrap_or_else(|| "news".into()))
}

#[component]
pub fn HydratedAdminMedia(query: ReadSignal<String>) -> Element {
    let seed_query = use_hook(|| query.read().clone());
    let initial = use_server_future(move || {
        let q = seed_query.clone();
        async move {
            match bucket_query(&q) {
                Ok(b) => read_media(b).await.unwrap_or(Err(LoadError::Unavailable)),
                Err(e) => Err(e),
            }
        }
    })?;
    let mut data = use_signal(|| {
        initial
            .read()
            .clone()
            .unwrap_or(Err(LoadError::Unavailable))
    });
    let mut observed = use_signal(|| query.read().clone());
    let mut generation = use_signal(|| 0u64);
    let mut form_generation = use_signal(|| 0u64);
    let mut busy = use_signal(|| false);
    let mut file = use_signal(|| None::<dioxus::html::FileData>);
    let mut notice = use_signal(|| None::<MediaMutationLoad>);
    let mut previous = use_signal(|| None::<MediaRequest>);
    let navigator = use_navigator();
    let refresh = use_callback(move |_: ()| {
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        let q = query();
        spawn(async move {
            let result = match bucket_query(&q) {
                Ok(b) => read_media(b).await.unwrap_or(Err(LoadError::Unavailable)),
                Err(e) => Err(e),
            };
            if *generation.peek() == ticket {
                data.set(result);
            }
        });
    });
    use_effect(move || {
        let q = query();
        if *observed.peek() != q {
            observed.set(q);
            notice.set(None);
            refresh.call(());
        }
    });
    let nav = use_callback(move |url: String| {
        let current = format!("/media?{}", query());
        if url == current {
            refresh.call(());
        } else {
            navigator.push(url);
        }
    });
    use_context_provider(|| AdminNavigation(nav));
    let choose = use_callback(move |event: FormEvent| {
        let selected = event.files().first().cloned();
        if selected
            .as_ref()
            .is_some_and(|v| v.size() > 25 * 1024 * 1024 || v.size() == 0)
        {
            file.set(None);
            notice.set(Some(MediaMutationLoad::Malformed));
        } else {
            file.set(selected);
        }
    });
    let submit = use_callback(move |event: FormEvent| {
        if *busy.peek() {
            return;
        }
        let values = event.values();
        let get = |key: &str| {
            values
                .iter()
                .find(|(name, _)| name == key)
                .and_then(|(_, v)| {
                    if let dioxus::html::FormValue::Text(v) = v {
                        Some(v.clone())
                    } else {
                        None
                    }
                })
        };
        let Some(_identity) = get("idempotency_key") else {
            return;
        };
        let deletion = get("key").zip(get("bucket"));
        let selected = file.peek().clone();
        if deletion.is_none() && selected.is_none() {
            notice.set(Some(MediaMutationLoad::Malformed));
            return;
        }
        busy.set(true);
        spawn(async move {
            let command = if let Some((key, bucket)) = deletion {
                MediaCommand::Delete { bucket, key }
            } else {
                let selected = selected.unwrap();
                match selected.read_bytes().await {
                    Ok(bytes) if !bytes.is_empty() && bytes.len() <= 25 * 1024 * 1024 => {
                        MediaCommand::Upload {
                            filename: selected.name(),
                            bytes: bytes.to_vec(),
                        }
                    }
                    _ => {
                        notice.set(Some(MediaMutationLoad::Malformed));
                        busy.set(false);
                        return;
                    }
                }
            };
            let mut request = MediaRequest {
                command,
                idempotency_key: format!("admin.media.{}", uuid::Uuid::new_v4()),
            };
            if let Some(last) = previous.peek().as_ref() {
                request.idempotency_key = if last.command == request.command {
                    last.idempotency_key.clone()
                } else {
                    format!("admin.media.{}", uuid::Uuid::new_v4())
                };
            }
            previous.set(Some(request.clone()));
            let result = change_media(request)
                .await
                .unwrap_or(Err(MediaFailure::Unavailable));
            let outcome = match result {
                Ok(value) => {
                    previous.set(None);
                    let next = *form_generation.peek() + 1;
                    form_generation.set(next);
                    file.set(None);
                    refresh.call(());
                    MediaMutationLoad::Committed(value)
                }
                Err(MediaFailure::Invalid | MediaFailure::Malformed) => {
                    MediaMutationLoad::Malformed
                }
                Err(MediaFailure::Forbidden) => MediaMutationLoad::Forbidden,
                Err(MediaFailure::Unauthenticated) => {
                    data.set(Err(LoadError::Unauthenticated));
                    MediaMutationLoad::Unauthenticated
                }
                Err(MediaFailure::Conflict) => {
                    MediaMutationLoad::Conflict("The object changed. Refresh and try again.".into())
                }
                Err(MediaFailure::Unavailable) => MediaMutationLoad::Unavailable,
            };
            notice.set(Some(outcome));
            busy.set(false);
        });
    });
    use_context_provider(|| MediaEvents {
        submit,
        choose,
        navigate: nav,
    });
    let bucket = if bucket_query(&query()).as_deref() == Ok("public") {
        MediaBucket::Public
    } else {
        MediaBucket::News
    };
    let load = match data() {
        Ok(list) if list.items.is_empty() => MediaLoad::Empty,
        Ok(list) => MediaLoad::Ready(list),
        Err(LoadError::Unauthenticated) => MediaLoad::Unauthenticated,
        Err(LoadError::Forbidden) => MediaLoad::Forbidden,
        Err(LoadError::InvalidQuery | LoadError::Malformed) => MediaLoad::Malformed,
        Err(_) => MediaLoad::Unavailable,
    };
    if let Err(error) = data() {
        crate::pages::news::hydrated::response_status(match error {
            LoadError::Unauthenticated => 401,
            LoadError::Forbidden => 403,
            LoadError::InvalidQuery => 400,
            _ => 502,
        });
    }
    rsx! {AdminAnalyticsShell{authenticated:data().is_ok(),current_path:"/media",title:"Media",fieldset{disabled:busy(),aria_busy:busy(),if busy(){p{role:"status","Updating media…"}}for epoch in [form_generation()] { MediaBody{key:"{epoch}",bucket,load:load.clone(),mutation:notice()} }}}}
}

#[cfg(feature = "server")]
pub type MediaProviderReadCallback = std::sync::Arc<
    dyn Fn(String, http::HeaderMap) -> Future<Result<AdminMediaList, LoadError>> + Send + Sync,
>;

#[cfg(feature = "server")]
pub type MediaProviderMutateCallback = std::sync::Arc<
    dyn Fn(
            MediaRequest,
            http::HeaderMap,
        ) -> Future<Result<AdminMediaMutationProjection, MediaFailure>>
        + Send
        + Sync,
>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn media_query_preserves_buckets_and_legacy_redirects_but_rejects_ambiguity() {
        assert_eq!(bucket_query(""), Ok("news".into()));
        assert_eq!(
            bucket_query("bucket=public&mutation=committed&key=a.pdf&size=1&deleted=false"),
            Ok("public".into())
        );
        for query in [
            "bucket=private",
            "bucket=news&bucket=public",
            "unknown=1",
            "deleted=yes",
            "size=-1",
        ] {
            assert_eq!(bucket_query(query), Err(LoadError::InvalidQuery));
        }
    }
}
