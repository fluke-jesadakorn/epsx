//! `/media` — authenticated media inventory with backend-authorized actions.
//!
//! Only a strict backend projection is rendered. Object URLs, previews,
//! credentials, storage-provider details, previews, search, and inferred
//! storage claims remain outside this UI boundary.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const MEDIA_PATH: &str = "/media";
const MAX_MEDIA_ITEMS: usize = 100;
const MAX_OBJECT_KEY_CHARS: usize = 1_024;
const MAX_OBJECT_KEY_BYTES: usize = 1_024;
const MAX_TIMESTAMP_CHARS: usize = 64;

pub const ADMIN_MEDIA_DATA_PARAM: &str = "data_admin_media";
pub const ADMIN_MEDIA_STATE_PARAM: &str = "data_admin_media_state";
pub const ADMIN_MEDIA_BUCKET_PARAM: &str = "admin_media_bucket";

pub const ADMIN_MEDIA_READY: &str = "ready";
pub const ADMIN_MEDIA_EMPTY: &str = "empty";
pub const ADMIN_MEDIA_FORBIDDEN: &str = "forbidden";
pub const ADMIN_MEDIA_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_MEDIA_MALFORMED: &str = "malformed";
pub const ADMIN_MEDIA_MUTATION_DATA_PARAM: &str = "data_admin_media_mutation";
pub const ADMIN_MEDIA_MUTATION_STATE_PARAM: &str = "data_admin_media_mutation_state";
pub const ADMIN_MEDIA_MUTATION_ERROR_PARAM: &str = "data_admin_media_mutation_error";
pub const ADMIN_MEDIA_MUTATION_COMMITTED: &str = "committed";
pub const ADMIN_MEDIA_MUTATION_CONFLICT: &str = "conflict";
pub const ADMIN_MEDIA_MUTATION_FORBIDDEN: &str = "forbidden";
pub const ADMIN_MEDIA_MUTATION_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_MEDIA_MUTATION_MALFORMED: &str = "malformed";

/// Deliberately excludes object URLs, media type guesses, provider metadata,
/// hashes, previews, and every field that could imply mutation access.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminMediaObject {
    pub key: String,
    pub size: i64,
    pub last_modified: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminMediaList {
    pub items: Vec<AdminMediaObject>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminMediaMutationProjection {
    pub bucket: String,
    pub key: String,
    pub size: Option<i64>,
    pub deleted: bool,
}

/// Decode the exact read projection and reject semantically unsafe values
/// before any backend-supplied object metadata reaches HTML.
pub fn decode_admin_media_projection(value: serde_json::Value) -> Option<AdminMediaList> {
    let projection: AdminMediaList = serde_json::from_value(value).ok()?;
    if projection.items.len() > MAX_MEDIA_ITEMS {
        return None;
    }

    let mut previous_key: Option<&str> = None;
    for item in &projection.items {
        if !bounded_control_free(&item.key, MAX_OBJECT_KEY_CHARS)
            || item.key.len() > MAX_OBJECT_KEY_BYTES
            || item.size < 0
            || !valid_optional_timestamp(item.last_modified.as_deref())
            || previous_key.is_some_and(|previous| previous >= item.key.as_str())
        {
            return None;
        }
        previous_key = Some(item.key.as_str());
    }

    Some(projection)
}

pub fn decode_admin_media_mutation(
    value: serde_json::Value,
) -> Option<AdminMediaMutationProjection> {
    let projection: AdminMediaMutationProjection = serde_json::from_value(value).ok()?;
    if !matches!(projection.bucket.as_str(), "news" | "public")
        || !bounded_control_free(&projection.key, MAX_OBJECT_KEY_CHARS)
        || projection.key.len() > MAX_OBJECT_KEY_BYTES
        || projection.size.is_some_and(|size| size < 0)
    {
        return None;
    }
    Some(projection)
}

fn bounded_control_free(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_optional_timestamp(value: Option<&str>) -> bool {
    value.is_none_or(|value| {
        !value.is_empty()
            && value.chars().count() <= MAX_TIMESTAMP_CHARS
            && !value.chars().any(char::is_control)
            && DateTime::parse_from_rfc3339(value).is_ok()
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MediaBucket {
    News,
    Public,
}

impl MediaBucket {
    fn from_ctx(ctx: &PageContext) -> Option<Self> {
        match ctx.params.get(ADMIN_MEDIA_BUCKET_PARAM).map(String::as_str) {
            None | Some("news") => Some(Self::News),
            Some("public") => Some(Self::Public),
            Some(_) => None,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::News => "news",
            Self::Public => "public",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::News => "News",
            Self::Public => "Public",
        }
    }

    fn href(self) -> String {
        format!("{MEDIA_PATH}?bucket={}", self.slug())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MediaLoad {
    Ready(AdminMediaList),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

fn media_load(ctx: &PageContext, bucket_valid: bool) -> MediaLoad {
    if !bucket_valid {
        return MediaLoad::Malformed;
    }

    let state = ctx.params.get(ADMIN_MEDIA_STATE_PARAM).map(String::as_str);
    match state {
        Some(ADMIN_MEDIA_READY) | Some(ADMIN_MEDIA_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_MEDIA_DATA_PARAM) else {
                return MediaLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_media_projection)
            else {
                return MediaLoad::Malformed;
            };

            match (state, projection.items.is_empty()) {
                (Some(ADMIN_MEDIA_READY), false) => MediaLoad::Ready(projection),
                (Some(ADMIN_MEDIA_EMPTY), true) => MediaLoad::Empty,
                _ => MediaLoad::Malformed,
            }
        }
        Some(ADMIN_MEDIA_FORBIDDEN) => MediaLoad::Forbidden,
        Some(ADMIN_MEDIA_MALFORMED) => MediaLoad::Malformed,
        Some(ADMIN_MEDIA_UNAVAILABLE) | None => MediaLoad::Unavailable,
        Some(_) => MediaLoad::Malformed,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MediaMutationLoad {
    Committed(AdminMediaMutationProjection),
    Conflict(String),
    Forbidden,
    Unavailable,
    Malformed,
}

fn media_mutation_load(ctx: &PageContext) -> Option<MediaMutationLoad> {
    match ctx
        .params
        .get(ADMIN_MEDIA_MUTATION_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_MEDIA_MUTATION_COMMITTED) => {
            let raw = ctx.params.get(ADMIN_MEDIA_MUTATION_DATA_PARAM)?;
            serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_media_mutation)
                .map(MediaMutationLoad::Committed)
                .or(Some(MediaMutationLoad::Malformed))
        }
        Some(ADMIN_MEDIA_MUTATION_CONFLICT) => Some(MediaMutationLoad::Conflict(
            ctx.params
                .get(ADMIN_MEDIA_MUTATION_ERROR_PARAM)
                .cloned()
                .unwrap_or_else(|| {
                    "The media object changed before the mutation completed.".into()
                }),
        )),
        Some(ADMIN_MEDIA_MUTATION_FORBIDDEN) => Some(MediaMutationLoad::Forbidden),
        Some(ADMIN_MEDIA_MUTATION_UNAVAILABLE) => Some(MediaMutationLoad::Unavailable),
        Some(ADMIN_MEDIA_MUTATION_MALFORMED) | Some(_) => Some(MediaMutationLoad::Malformed),
        None => None,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Media");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private media inventory".to_string()),
                return_url: Some(MEDIA_PATH.to_string()),
                RenderMedia { ctx: ctx.clone() }
            }
        },
    )
}

#[component]
fn RenderMedia(ctx: PageContext) -> Element {
    let parsed_bucket = MediaBucket::from_ctx(&ctx);
    let bucket = parsed_bucket.unwrap_or(MediaBucket::News);
    let load = media_load(&ctx, parsed_bucket.is_some());

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            PageHeader {
                title: "Media".to_string(),
                subtitle: Some("Review backend-authoritative object metadata".to_string()),
                icon: Some("image".to_string()),
                gradient: Some(PageGradient::Info),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            MediaBucketNav { selected: bucket }
            if let Some(mutation) = media_mutation_load(&ctx) {
                MediaMutationNotice { mutation }
            }
            if matches!(&load, MediaLoad::Ready(_) | MediaLoad::Empty) {
                MediaMutationControls { bucket }
            }
            match load {
                MediaLoad::Ready(projection) => rsx! {
                    MediaReady { projection, bucket }
                },
                MediaLoad::Empty => rsx! {
                    MediaEmpty { bucket }
                },
                MediaLoad::Forbidden => rsx! {
                    MediaProblem {
                        state: ADMIN_MEDIA_FORBIDDEN,
                        title: "Media access was denied".to_string(),
                        detail: "The backend did not authorize this session to read the selected media inventory.".to_string(),
                        retry_href: bucket.href(),
                    }
                },
                MediaLoad::Unavailable => rsx! {
                    MediaProblem {
                        state: ADMIN_MEDIA_UNAVAILABLE,
                        title: "Media inventory is unavailable".to_string(),
                        detail: "The storage backend could not provide an authoritative response. No object metadata is being shown.".to_string(),
                        retry_href: bucket.href(),
                    }
                },
                MediaLoad::Malformed => rsx! {
                    MediaProblem {
                        state: ADMIN_MEDIA_MALFORMED,
                        title: "Media data could not be verified".to_string(),
                        detail: "The requested bucket or backend response did not match the read-only media contract. No object metadata is being shown.".to_string(),
                        retry_href: bucket.href(),
                    }
                },
            }
        }
    }
}

#[component]
fn MediaMutationNotice(mutation: MediaMutationLoad) -> Element {
    match mutation {
        MediaMutationLoad::Committed(projection) => rsx! {
            section {
                class: "mb-5 rounded-2xl border border-emerald-500/30 bg-emerald-500/5 p-6",
                role: "status",
                "data-admin-media-mutation-state": ADMIN_MEDIA_MUTATION_COMMITTED,
                h2 { class: "text-lg font-semibold text-foreground", if projection.deleted { "Media object deleted" } else { "Media object uploaded" } }
                p { class: "mt-2 break-all text-sm text-muted-foreground", "{projection.bucket}/{projection.key}" }
            }
        },
        MediaMutationLoad::Conflict(detail) => {
            rsx! { MediaMutationProblem { state: ADMIN_MEDIA_MUTATION_CONFLICT, detail } }
        }
        MediaMutationLoad::Forbidden => {
            rsx! { MediaMutationProblem { state: ADMIN_MEDIA_MUTATION_FORBIDDEN, detail: "The backend denied this media mutation. No storage state is being inferred.".to_string() } }
        }
        MediaMutationLoad::Unavailable => {
            rsx! { MediaMutationProblem { state: ADMIN_MEDIA_MUTATION_UNAVAILABLE, detail: "The storage backend did not provide an authoritative mutation result.".to_string() } }
        }
        MediaMutationLoad::Malformed => {
            rsx! { MediaMutationProblem { state: ADMIN_MEDIA_MUTATION_MALFORMED, detail: "The storage mutation response did not match the strict media contract.".to_string() } }
        }
    }
}

#[component]
fn MediaMutationProblem(state: &'static str, detail: String) -> Element {
    rsx! {
        section {
            class: "mb-5 rounded-2xl border border-amber-500/30 bg-amber-500/5 p-6",
            role: "alert",
            "data-admin-media-mutation-state": state,
            h2 { class: "text-lg font-semibold text-foreground", "Media mutation: {state}" }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn MediaBucketNav(selected: MediaBucket) -> Element {
    rsx! {
        nav {
            class: "mb-5 flex flex-wrap gap-2",
            aria_label: "Media bucket",
            for bucket in [MediaBucket::News, MediaBucket::Public] {
                a {
                    class: if selected == bucket { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                    href: bucket.href(),
                    aria_current: (selected == bucket).then_some("page"),
                    "{bucket.label()}"
                }
            }
        }
    }
}

#[component]
fn MediaMutationControls(bucket: MediaBucket) -> Element {
    rsx! {
        section { class: "mb-5 grid gap-4 rounded-2xl border border-border/30 bg-card p-5 md:grid-cols-[1fr_auto] md:items-end",
            div {
                h2 { class: "text-lg font-semibold text-foreground", "Media mutations" }
                p { class: "mt-1 text-sm leading-6 text-muted-foreground",
                    "Upload is written to the backend public bucket. Deletion is available per verified object row."
                }
            }
            form {
                method: "post",
                action: "/media/upload",
                enctype: "multipart/form-data",
                class: "grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end",
                label { class: "space-y-2 text-sm font-medium",
                    span { "Public file" }
                    input { class: "input w-full", r#type: "file", name: "file", required: true }
                }
                input { r#type: "hidden", name: "idempotency_key", value: format!("admin.media.upload.{}", uuid::Uuid::new_v4()) }
                button { class: "btn btn-primary", r#type: "submit", "Upload file" }
            }
            if bucket != MediaBucket::Public {
                p { class: "text-xs text-muted-foreground sm:col-span-2",
                    "The upload form targets Public; switch buckets after a successful upload to inspect the resulting inventory."
                }
            }
        }
    }
}

#[component]
fn MediaReady(projection: AdminMediaList, bucket: MediaBucket) -> Element {
    let item_count = projection.items.len();
    rsx! {
        section {
            class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            aria_labelledby: "admin-media-inventory-title",
            "data-admin-media-state": ADMIN_MEDIA_READY,
            "data-admin-media-bucket": bucket.slug(),
            div { class: "h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ffb237]" }
            div { class: "flex flex-wrap items-start justify-between gap-4 p-5",
                div {
                    h2 { id: "admin-media-inventory-title", class: "text-lg font-semibold text-foreground",
                        "{bucket.label()} object metadata"
                    }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "{item_count} metadata records in this bounded response"
                    }
                }
                p { class: "max-w-xl text-xs leading-5 text-muted-foreground",
                    "This inventory is a bounded first page of up to 100 objects; continuation is unavailable. URLs, previews, and storage credentials are not displayed."
                }
            }
            if item_count == MAX_MEDIA_ITEMS {
                p {
                    class: "mx-5 mb-5 rounded-xl border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-900 dark:text-amber-200",
                    role: "status",
                    "100 objects are shown, reaching the bounded first-page limit. Additional objects may exist because continuation is unavailable."
                }
            }
            div {
                class: "hidden grid-cols-12 gap-4 border-t border-border/30 bg-muted/20 px-5 py-3 text-xs font-semibold uppercase tracking-wide text-muted-foreground md:grid",
                aria_hidden: "true",
                span { class: "col-span-7", "Object key" }
                span { class: "col-span-2 text-right", "Size" }
                span { class: "col-span-3", "Last modified" }
            }
            ul {
                class: "divide-y divide-border/30",
                aria_label: format!("{} media objects", bucket.label()),
                for item in projection.items {
                    MediaRow { item, bucket }
                }
            }
        }
    }
}

#[component]
fn MediaRow(item: AdminMediaObject, bucket: MediaBucket) -> Element {
    let size = readable_bytes(item.size);
    rsx! {
        li { class: "grid gap-4 p-5 md:grid-cols-12 md:items-center",
            div { class: "min-w-0 md:col-span-7",
                span { class: "sr-only", "Object key: " }
                p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground md:hidden", aria_hidden: "true", "Object key" }
                p { class: "break-all text-sm font-semibold text-foreground", "{item.key}" }
            }
            div { class: "md:col-span-2 md:text-right",
                span { class: "sr-only", "Size: " }
                p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground md:hidden", aria_hidden: "true", "Size" }
                p { class: "text-sm text-foreground", "{size}" }
            }
            div { class: "md:col-span-3",
                span { class: "sr-only", "Last modified: " }
                p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground md:hidden", aria_hidden: "true", "Last modified" }
                if let Some(last_modified) = item.last_modified {
                    time { class: "break-words text-sm text-muted-foreground", datetime: last_modified.clone(), "{last_modified}" }
                } else {
                    span { class: "text-sm text-muted-foreground", "Not reported" }
                }
            }
            form { method: "post", action: "/media", class: "md:col-span-12 md:flex md:justify-end",
                input { r#type: "hidden", name: "bucket", value: bucket.slug() }
                input { r#type: "hidden", name: "key", value: item.key.clone() }
                input { r#type: "hidden", name: "idempotency_key", value: format!("admin.media.delete.{}", uuid::Uuid::new_v4()) }
                button { class: "btn btn-sm btn-outline", r#type: "submit", "Delete object" }
            }
        }
    }
}

fn readable_bytes(size: i64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if size < 1_024 {
        return format!("{size} B");
    }

    let mut value = size as f64;
    let mut unit = 0;
    while value >= 1_024.0 && unit < UNITS.len() - 1 {
        value /= 1_024.0;
        unit += 1;
    }
    if value >= 10.0 || value.fract().abs() < 0.05 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[component]
fn MediaEmpty(bucket: MediaBucket) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-border/30 bg-card p-10 text-center shadow-xl",
            role: "status",
            "data-admin-media-state": ADMIN_MEDIA_EMPTY,
            "data-admin-media-bucket": bucket.slug(),
            Icon { name: "image".to_string(), size: Some(30) }
            h2 { class: "mt-4 text-xl font-semibold text-foreground", "No object metadata found" }
            p { class: "mx-auto mt-2 max-w-xl text-sm leading-6 text-muted-foreground",
                "The backend returned no metadata records for the {bucket.label()} bucket. The inventory is bounded to a first page of up to 100 objects, and continuation is unavailable."
            }
            a { class: "btn btn-outline mt-5", href: MEDIA_PATH, "Reset media view" }
        }
    }
}

#[component]
fn MediaProblem(state: &'static str, title: String, detail: String, retry_href: String) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-border/30 bg-card p-10 text-center shadow-xl",
            role: if state == ADMIN_MEDIA_FORBIDDEN { "alert" } else { "status" },
            "data-admin-media-state": state,
            Icon { name: "shield".to_string(), size: Some(30) }
            h2 { class: "mt-4 text-xl font-semibold text-foreground", "{title}" }
            p { class: "mx-auto mt-2 max-w-2xl text-sm leading-6 text-muted-foreground", "{detail}" }
            div { class: "mt-6 flex flex-wrap justify-center gap-3",
                a { class: "btn btn-primary", href: retry_href, "Try again" }
                a { class: "btn btn-outline", href: MEDIA_PATH, "Reset media view" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-1".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: MEDIA_PATH.to_string(),
            ..Default::default()
        }
    }

    fn with_state(state: &str, payload: Option<&str>) -> PageContext {
        let mut ctx = signed_in_ctx();
        ctx.params
            .insert(ADMIN_MEDIA_STATE_PARAM.to_string(), state.to_string());
        if let Some(payload) = payload {
            ctx.params
                .insert(ADMIN_MEDIA_DATA_PARAM.to_string(), payload.to_string());
        }
        ctx
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn ready_payload() -> String {
        serde_json::to_string(&AdminMediaList {
            items: vec![
                AdminMediaObject {
                    key: "banners/launch.webp".to_string(),
                    size: 1_536,
                    last_modified: Some("2026-07-22T12:00:00Z".to_string()),
                },
                AdminMediaObject {
                    key: "documents/terms.pdf".to_string(),
                    size: 2_048,
                    last_modified: None,
                },
            ],
        })
        .unwrap()
    }

    fn assert_unsupported_actions_absent(rendered: &str) {
        for forbidden in [
            "Copy URL",
            "Open file",
            "Preview",
            "Search",
            "Grid view",
            "List view",
            "onclick=",
            "javascript:",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "forbidden media action leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn signed_out_route_keeps_media_state_private() {
        let rendered = html(&PageContext {
            path: MEDIA_PATH.to_string(),
            ..Default::default()
        });

        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("href=\"/auth?return_url=%2Fmedia\""));
        assert!(!rendered.contains("data-admin-media-state"));
        assert!(!rendered.contains("Media bucket"));
        assert!(!rendered.contains("object metadata"));
    }

    #[test]
    fn ready_inventory_is_redacted_accessible_responsive_and_uses_native_links() {
        let payload = ready_payload();
        let mut ctx = with_state(ADMIN_MEDIA_READY, Some(&payload));
        ctx.params
            .insert(ADMIN_MEDIA_BUCKET_PARAM.to_string(), "public".to_string());
        let rendered = html(&ctx);

        assert!(rendered.contains("data-admin-media-state=\"ready\""));
        assert!(rendered.contains("data-admin-media-bucket=\"public\""));
        assert!(rendered.contains("banners/launch.webp"));
        assert!(rendered.contains("1.5 KiB"));
        assert!(rendered.contains("2 KiB"));
        assert!(rendered.contains("datetime=\"2026-07-22T12:00:00Z\""));
        assert!(rendered.contains("Not reported"));
        assert!(rendered.contains("Object key: "));
        assert!(rendered.contains("Size: "));
        assert!(rendered.contains("Last modified: "));
        assert!(rendered.contains("md:grid-cols-12"));
        assert!(rendered.contains("href=\"/media?bucket=news\""));
        assert!(rendered.contains("href=\"/media?bucket=public\""));
        assert!(rendered.contains("aria-current=\"page\""));
        assert!(rendered.contains("bounded first page of up to 100 objects"));
        assert!(rendered.contains("continuation is unavailable"));
        assert!(rendered.contains("Upload file"));
        assert!(rendered.contains("Delete object"));
        assert!(rendered.contains("<form"));
        assert_unsupported_actions_absent(&rendered);
        for redacted in ["https://", "presigned", "mime_type", "etag"] {
            assert!(
                !rendered.contains(redacted),
                "sensitive field leaked: {redacted}"
            );
        }
    }

    #[test]
    fn authoritative_empty_is_distinct_and_defaults_to_news() {
        let rendered = html(&with_state(ADMIN_MEDIA_EMPTY, Some(r#"{"items":[]}"#)));

        assert!(rendered.contains("data-admin-media-state=\"empty\""));
        assert!(rendered.contains("data-admin-media-bucket=\"news\""));
        assert!(rendered.contains("returned no metadata records for the News bucket"));
        assert!(!rendered.contains("inventory is unavailable"));
        assert!(rendered.contains("Upload file"));
        assert_unsupported_actions_absent(&rendered);
    }

    #[test]
    fn explicit_problem_states_have_native_retry_and_reset_paths() {
        for (state, title) in [
            (ADMIN_MEDIA_FORBIDDEN, "Media access was denied"),
            (ADMIN_MEDIA_UNAVAILABLE, "Media inventory is unavailable"),
            (ADMIN_MEDIA_MALFORMED, "Media data could not be verified"),
        ] {
            let rendered = html(&with_state(state, None));
            assert!(rendered.contains(&format!("data-admin-media-state=\"{state}\"")));
            assert!(rendered.contains(title));
            assert!(rendered.contains("href=\"/media?bucket=news\""));
            assert!(rendered.contains("href=\"/media\""));
            assert!(!rendered.contains("<form"));
            assert!(!rendered.contains("<button"));
            assert_unsupported_actions_absent(&rendered);
        }
    }

    #[test]
    fn hostile_unknown_and_semantically_malformed_payloads_fail_closed() {
        let payloads = [
            r#"{"items":[{"key":"a","size":1,"last_modified":null,"url":"https://secret.example/object"}]}"#,
            r#"{"items":[{"key":"b","size":1,"last_modified":null},{"key":"a","size":2,"last_modified":null}]}"#,
            r#"{"items":[{"key":"a","size":1,"last_modified":null},{"key":"a","size":2,"last_modified":null}]}"#,
            r#"{"items":[{"key":"bad\nkey","size":1,"last_modified":null}]}"#,
            r#"{"items":[{"key":"a","size":-1,"last_modified":null}]}"#,
            r#"{"items":[{"key":"a","size":1,"last_modified":"yesterday"}]}"#,
            r#"{"items":[{"key":" padded ","size":1,"last_modified":null}]}"#,
            r#"{"items":[],"total":7}"#,
        ];

        for payload in payloads {
            let rendered = html(&with_state(ADMIN_MEDIA_READY, Some(payload)));
            assert!(rendered.contains("data-admin-media-state=\"malformed\""));
            assert!(!rendered.contains("secret.example"));
            assert!(!rendered.contains("bad\nkey"));
            assert!(!rendered.contains("<form"));
            assert_unsupported_actions_absent(&rendered);
        }

        let too_many = AdminMediaList {
            items: (0..=MAX_MEDIA_ITEMS)
                .map(|index| AdminMediaObject {
                    key: format!("{index:03}"),
                    size: 0,
                    last_modified: None,
                })
                .collect(),
        };
        let rendered = html(&with_state(
            ADMIN_MEDIA_READY,
            Some(&serde_json::to_string(&too_many).unwrap()),
        ));
        assert!(rendered.contains("data-admin-media-state=\"malformed\""));

        let oversized_utf8_key = AdminMediaList {
            items: vec![AdminMediaObject {
                key: "é".repeat(600),
                size: 1,
                last_modified: None,
            }],
        };
        let rendered = html(&with_state(
            ADMIN_MEDIA_READY,
            Some(&serde_json::to_string(&oversized_utf8_key).unwrap()),
        ));
        assert!(rendered.contains("data-admin-media-state=\"malformed\""));

        let long_timestamp = AdminMediaList {
            items: vec![AdminMediaObject {
                key: "long-time".to_string(),
                size: 1,
                last_modified: Some(format!("2026-07-22T12:00:00.{}Z", "1".repeat(80))),
            }],
        };
        let rendered = html(&with_state(
            ADMIN_MEDIA_READY,
            Some(&serde_json::to_string(&long_timestamp).unwrap()),
        ));
        assert!(rendered.contains("data-admin-media-state=\"malformed\""));
    }

    #[test]
    fn invalid_bucket_unknown_state_and_state_payload_mismatch_are_malformed() {
        let payload = ready_payload();
        let mut invalid_bucket = with_state(ADMIN_MEDIA_READY, Some(&payload));
        invalid_bucket.params.insert(
            ADMIN_MEDIA_BUCKET_PARAM.to_string(),
            "chat<script>".to_string(),
        );
        let rendered = html(&invalid_bucket);
        assert!(rendered.contains("data-admin-media-state=\"malformed\""));
        assert!(!rendered.contains("chat&lt;script"));
        assert!(!rendered.contains("banners/launch.webp"));

        let rendered = html(&with_state("invented", None));
        assert!(rendered.contains("data-admin-media-state=\"malformed\""));

        let rendered = html(&with_state(ADMIN_MEDIA_EMPTY, Some(&payload)));
        assert!(rendered.contains("data-admin-media-state=\"malformed\""));
        assert!(!rendered.contains("banners/launch.webp"));
    }

    #[test]
    fn malformed_public_inventory_retries_public_and_resets_to_news() {
        let mut ctx = with_state(ADMIN_MEDIA_MALFORMED, None);
        ctx.params
            .insert(ADMIN_MEDIA_BUCKET_PARAM.to_string(), "public".to_string());
        let rendered = html(&ctx);

        assert!(rendered.contains("data-admin-media-state=\"malformed\""));
        assert!(rendered.contains("href=\"/media?bucket=public\""));
        assert!(rendered.contains("href=\"/media\""));
    }

    #[test]
    fn first_page_limit_has_an_explicit_continuation_warning() {
        let full_page = AdminMediaList {
            items: (0..MAX_MEDIA_ITEMS)
                .map(|index| AdminMediaObject {
                    key: format!("object-{index:03}"),
                    size: index as i64,
                    last_modified: None,
                })
                .collect(),
        };
        let rendered = html(&with_state(
            ADMIN_MEDIA_READY,
            Some(&serde_json::to_string(&full_page).unwrap()),
        ));

        assert!(rendered.contains("100 objects are shown"));
        assert!(rendered.contains("Additional objects may exist"));
        assert!(rendered.contains("continuation is unavailable"));
        assert!(rendered.contains("Delete object"));
        assert_unsupported_actions_absent(&rendered);
    }

    #[test]
    fn mutation_projection_is_strict_and_problem_states_are_truthful() {
        let valid = serde_json::json!({
            "bucket": "news",
            "key": "images/launch.webp",
            "size": 42,
            "deleted": false
        });
        assert!(decode_admin_media_mutation(valid.clone()).is_some());

        let mut unknown = valid.clone();
        unknown["url"] = serde_json::json!("https://objects.example/secret");
        assert!(decode_admin_media_mutation(unknown).is_none());
        let mut private_bucket = valid.clone();
        private_bucket["bucket"] = serde_json::json!("chat");
        assert!(decode_admin_media_mutation(private_bucket).is_none());
        let mut negative_size = valid;
        negative_size["size"] = serde_json::json!(-1);
        assert!(decode_admin_media_mutation(negative_size).is_none());

        let mut conflict = with_state(ADMIN_MEDIA_EMPTY, Some(r#"{"items":[]}"#));
        conflict.params.insert(
            ADMIN_MEDIA_MUTATION_STATE_PARAM.to_string(),
            ADMIN_MEDIA_MUTATION_CONFLICT.to_string(),
        );
        conflict.params.insert(
            ADMIN_MEDIA_MUTATION_ERROR_PARAM.to_string(),
            "version conflict".to_string(),
        );
        let rendered = html(&conflict);
        assert!(rendered.contains("data-admin-media-mutation-state=\"conflict\""));
        assert!(rendered.contains("version conflict"));

        let malformed = with_state(ADMIN_MEDIA_EMPTY, Some(r#"{"items":[]}"#));
        let mut malformed = malformed;
        malformed.params.insert(
            ADMIN_MEDIA_MUTATION_STATE_PARAM.to_string(),
            ADMIN_MEDIA_MUTATION_COMMITTED.to_string(),
        );
        malformed.params.insert(
            ADMIN_MEDIA_MUTATION_DATA_PARAM.to_string(),
            r#"{"bucket":"news","key":"bad","size":-1,"deleted":false}"#.to_string(),
        );
        assert!(html(&malformed).contains("data-admin-media-mutation-state=\"malformed\""));
    }
}
