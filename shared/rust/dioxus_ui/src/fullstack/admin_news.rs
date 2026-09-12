//! Admin content UI data stays typed from the protected adapter to hydration.
use super::{
    admin::{AdminAnalyticsShell, AdminNavigation},
    LoadError,
};
use crate::pages::admin_pages::news::*;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NewsPage {
    List,
    Create,
    Edit(String),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewsData {
    pub list: Option<AdminNewsList>,
    pub article: Option<AdminNewsEditorProjection>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewsFields {
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub summary: Option<String>,
    pub cover_image_url: Option<String>,
    pub tags: Vec<String>,
    pub status: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NewsCommand {
    Save {
        id: Option<String>,
        fields: NewsFields,
        version: Option<String>,
    },
    Transition {
        id: String,
        operation: String,
        version: String,
    },
    Delete {
        id: String,
        version: String,
    },
    Upload {
        filename: String,
        bytes: Vec<u8>,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewsRequest {
    pub command: NewsCommand,
    pub idempotency_key: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewsChanged {
    pub article: Option<AdminNewsEditorProjection>,
    pub image_url: Option<String>,
    pub deleted: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NewsFailure {
    Invalid,
    Conflict,
    Forbidden,
    Unauthenticated,
    Unavailable,
    Malformed,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct NewsProvider {
    pub read: NewsProviderReadCallback,
    pub mutate: NewsProviderMutateCallback,
}
#[server(prefix = "/_server/admin", endpoint = "news_read")]
pub async fn read_news(
    page: NewsPage,
    query: String,
) -> Result<Result<NewsData, LoadError>, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<NewsProvider>,
        _,
    >()
    .await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(page, query, headers).await)
}
#[server(prefix = "/_server/admin", endpoint = "news_change")]
pub async fn change_news(
    request: NewsRequest,
) -> Result<Result<NewsChanged, NewsFailure>, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<NewsProvider>,
        _,
    >()
    .await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.mutate)(request, headers).await)
}
#[derive(Clone, Copy)]
struct NewsEvents {
    submit: EventHandler<FormEvent>,
    choose: EventHandler<FormEvent>,
    navigate: EventHandler<String>,
}
pub fn submit(event: FormEvent) {
    if let Some(h) = try_consume_context::<NewsEvents>() {
        event.prevent_default();
        h.submit.call(event);
    }
}
pub fn choose_file(event: FormEvent) {
    if let Some(h) = try_consume_context::<NewsEvents>() {
        h.choose.call(event);
    }
}
pub fn follow(event: MouseEvent, url: String) {
    if let Some(h) = try_consume_context::<NewsEvents>() {
        event.prevent_default();
        h.navigate.call(url);
    }
}
fn form_command(
    values: &[(String, dioxus::html::FormValue)],
    page: &NewsPage,
) -> Option<(NewsCommand, String)> {
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
    let identity = get("idempotency_key")?;
    let command = if let Some(id) = get("id") {
        NewsCommand::Delete {
            id,
            version: get("if_match")?,
        }
    } else if let Some(operation) = get("operation") {
        let NewsPage::Edit(id) = page else {
            return None;
        };
        NewsCommand::Transition {
            id: id.clone(),
            operation,
            version: get("if_match")?,
        }
    } else {
        NewsCommand::Save {
            id: if let NewsPage::Edit(id) = page {
                Some(id.clone())
            } else {
                None
            },
            version: get("if_match"),
            fields: NewsFields {
                title: get("title")?,
                slug: get("slug"),
                content: get("content")?,
                summary: get("summary").filter(|v| !v.trim().is_empty()),
                cover_image_url: get("cover_image_url").filter(|v| !v.trim().is_empty()),
                tags: get("tags")
                    .unwrap_or_default()
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect(),
                status: get("status"),
            },
        }
    };
    Some((command, identity))
}
#[component]
pub fn HydratedAdminNews(page: NewsPage, query: ReadSignal<String>) -> Element {
    let page = use_hook(|| page);
    let seed = use_hook(|| (page.clone(), query()));
    let initial = use_server_future(move || {
        let (p, q) = seed.clone();
        async move { read_news(p, q).await.unwrap_or(Err(LoadError::Unavailable)) }
    })?;
    let mut data = use_signal(|| {
        initial
            .read()
            .clone()
            .unwrap_or(Err(LoadError::Unavailable))
    });
    let mut observed = use_signal(|| query.read().clone());
    let mut generation = use_signal(|| 0u64);
    let mut identity_generation = use_signal(|| 0u64);
    let mut pending = use_signal(|| false);
    let mut previous = use_signal(|| None::<NewsRequest>);
    let mut notice = use_signal(|| None::<String>);
    let mut cover = use_signal(|| None::<String>);
    let mut file = use_signal(|| None::<dioxus::html::FileData>);
    let current = use_signal(|| page.clone());
    let nav = use_navigator();
    let refresh = use_callback(move |_: ()| {
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        let q = query();
        let p = current();
        spawn(async move {
            let result = read_news(p, q).await.unwrap_or(Err(LoadError::Unavailable));
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
    let navigate = use_callback(move |url: String| {
        if url == format!("/news?{}", query()) {
            refresh.call(());
        } else {
            nav.push(url);
        }
    });
    use_context_provider(|| AdminNavigation(navigate));
    let choose = use_callback(move |event: FormEvent| {
        let selected = event.files().first().cloned();
        if selected
            .as_ref()
            .is_some_and(|v| v.size() == 0 || v.size() > 25 * 1024 * 1024)
        {
            file.set(None);
            notice.set(Some("Choose an image up to 25MB.".into()));
        } else {
            file.set(selected);
        }
    });
    let submit = use_callback(move |event: FormEvent| {
        if *pending.peek() {
            return;
        }
        let values = event.values();
        let upload = values.iter().any(|(name, _)| name == "article_id");
        let identity = values
            .iter()
            .find(|(n, _)| n == "idempotency_key")
            .and_then(|(_, v)| {
                if let dioxus::html::FormValue::Text(v) = v {
                    Some(v.clone())
                } else {
                    None
                }
            });
        let parsed = form_command(&values, &current());
        let selected = file.peek().clone();
        if (!upload && parsed.is_none()) || (upload && (selected.is_none() || identity.is_none())) {
            notice.set(Some("Check the required article fields.".into()));
            return;
        }
        pending.set(true);
        spawn(async move {
            let (command, _idempotency_key) = if upload {
                let f = selected.unwrap();
                match f.read_bytes().await {
                    Ok(bytes) if !bytes.is_empty() && bytes.len() <= 25 * 1024 * 1024 => (
                        NewsCommand::Upload {
                            filename: f.name(),
                            bytes: bytes.to_vec(),
                        },
                        identity.unwrap(),
                    ),
                    _ => {
                        notice.set(Some("The image could not be read.".into()));
                        pending.set(false);
                        return;
                    }
                }
            } else {
                parsed.unwrap()
            };
            let mut request = NewsRequest {
                command,
                idempotency_key: format!("admin.news.{}", uuid::Uuid::new_v4()),
            };
            if let Some(last) = previous.peek().as_ref() {
                request.idempotency_key = if last.command == request.command {
                    last.idempotency_key.clone()
                } else {
                    format!("admin.news.{}", uuid::Uuid::new_v4())
                };
            }
            previous.set(Some(request.clone()));
            match change_news(request)
                .await
                .unwrap_or(Err(NewsFailure::Unavailable))
            {
                Ok(changed) => {
                    previous.set(None);
                    if let Some(url) = changed.image_url {
                        cover.set(Some(url));
                        notice.set(Some(
                            "Image uploaded. Save the article to use this cover.".into(),
                        ));
                    } else if let Some(article) = changed.article {
                        notice.set(Some("Article saved.".into()));
                        if matches!(current(), NewsPage::Create) {
                            nav.replace(format!("/news/{}/edit", article.id));
                        } else {
                            data.set(Ok(NewsData {
                                list: None,
                                article: Some(article),
                            }));
                            let next = *identity_generation.peek() + 1;
                            identity_generation.set(next);
                        }
                    } else if changed.deleted {
                        notice.set(Some("Article deleted.".into()));
                        refresh.call(());
                    }
                }
                Err(error) => {
                    if error == NewsFailure::Unauthenticated {
                        data.set(Err(LoadError::Unauthenticated));
                    }
                    notice.set(Some(match error{NewsFailure::Conflict=>"The article changed. Your inputs are preserved; reload before retrying.",NewsFailure::Forbidden=>"The backend denied this action.",NewsFailure::Unauthenticated=>"Your session expired. Sign in again.",NewsFailure::Invalid|NewsFailure::Malformed=>"The article could not be verified. Check your input.",NewsFailure::Unavailable=>"The content service is unavailable. Try the action again."}.into()));
                }
            }
            pending.set(false);
        });
    });
    use_context_provider(|| NewsEvents {
        submit,
        choose,
        navigate,
    });
    let fields = url::form_urlencoded::parse(query().as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect::<std::collections::HashMap<_, _>>();
    let filters = NewsFilters {
        page: fields.get("page").and_then(|v| v.parse().ok()).unwrap_or(1),
        status: fields
            .get("status")
            .cloned()
            .unwrap_or_else(|| "all".into()),
    };
    if let Err(error) = data() {
        crate::pages::news::hydrated::response_status(match error {
            LoadError::Unauthenticated => 401,
            LoadError::Forbidden => 403,
            LoadError::InvalidQuery => 400,
            LoadError::NotFound => 404,
            _ => 502,
        });
    }
    rsx! {AdminAnalyticsShell{authenticated:data().is_ok(),current_path:"/news",title:"News",fieldset{disabled:pending(),aria_busy:pending(),
        if pending(){p{role:"status","Saving…"}}
        if let Some(message)=notice(){p{class:"m-4 rounded-xl border border-border p-4",role:"status","{message}"}}
        match data(){
            Ok(snapshot)=>match page{
                NewsPage::List=>{let load=match snapshot.list{Some(list)if list.articles.is_empty()&&list.total==0=>NewsLoad::Empty,Some(list)=>NewsLoad::Ready(list),None=>NewsLoad::Malformed};rsx!{NewsListBody{filters,load,mutation:None}}},
                NewsPage::Create=>rsx!{div{class:"p-4 md:p-8",NewsEditor{route:NewsRoute::Create,projection:None,route_reference:None,image_url:cover()}}},
                NewsPage::Edit(id)=>rsx!{div{class:"p-4 md:p-8",NewsEditor{key:"{identity_generation}",route:NewsRoute::Edit,projection:snapshot.article,route_reference:Some(id),image_url:cover()}}},
            },
            Err(error)=>rsx!{section{class:"p-6",p{role:"alert","{error.message()}"}button{class:"btn btn-primary",r#type:"button",onclick:move |_|refresh.call(()),"Try again"}}},
        }
    }}}
}

#[component]
pub fn NewsIdentity(prefix: String) -> Element {
    let identity = use_server_cached(|| format!("{prefix}.{}", uuid::Uuid::new_v4()));
    rsx! {input{r#type:"hidden",name:"idempotency_key",value:identity}}
}

#[cfg(feature = "server")]
pub type NewsProviderReadCallback = std::sync::Arc<
    dyn Fn(NewsPage, String, http::HeaderMap) -> Future<Result<NewsData, LoadError>> + Send + Sync,
>;

#[cfg(feature = "server")]
pub type NewsProviderMutateCallback = std::sync::Arc<
    dyn Fn(NewsRequest, http::HeaderMap) -> Future<Result<NewsChanged, NewsFailure>> + Send + Sync,
>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editor_submission_keeps_version_and_treats_empty_cover_as_absent() {
        let values = [
            ("title", "Research update"),
            ("content", "# Research"),
            ("summary", ""),
            ("cover_image_url", ""),
            ("tags", "a, b"),
            ("status", "draft"),
            ("if_match", "2026-09-10T00:00:00Z"),
            ("idempotency_key", "test-request"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), dioxus::html::FormValue::Text(v.into())))
        .collect::<Vec<_>>();
        let (
            NewsCommand::Save {
                id,
                fields,
                version,
            },
            key,
        ) = form_command(&values, &NewsPage::Edit("article".into())).unwrap()
        else {
            panic!("save command")
        };
        assert_eq!(id.as_deref(), Some("article"));
        assert_eq!(fields.cover_image_url, None);
        assert_eq!(fields.tags, ["a", "b"]);
        assert_eq!(version.as_deref(), Some("2026-09-10T00:00:00Z"));
        assert_eq!(key, "test-request");
    }
}
