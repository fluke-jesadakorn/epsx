//! Navigation shared by hydrated applications and standalone SSR renderers.
use dioxus::prelude::*;

/// Keep real hrefs in SSR; only app routes are eligible for client navigation.
pub fn local_route(href: &str) -> bool {
    href.starts_with('/')
        && !href.starts_with("//")
        && !href.contains(['\\', '\r', '\n'])
        && !["/api/", "/_server/", "/public/", "/runtime/", "/assets/"]
            .iter()
            .any(|prefix| href.starts_with(prefix))
}

#[derive(Clone, Copy)]
struct AppOrigin(Signal<Option<String>>);

fn internal_destination(href: &str, origin: Option<&str>, current: Option<&str>) -> Option<String> {
    if local_route(href) {
        return Some(href.to_owned());
    }
    if href.starts_with('?') {
        let path = current?.split(['?', '#']).next()?;
        return local_route(path).then(|| format!("{path}{href}"));
    }
    let url = url::Url::parse(href).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.origin().ascii_serialization() != origin?
    {
        return None;
    }
    let mut path = url.path().to_owned();
    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }
    if let Some(hash) = url.fragment() {
        path.push('#');
        path.push_str(hash);
    }
    local_route(&path).then_some(path)
}

#[component]
pub fn AppLink(
    #[props(default)] href: String,
    #[props(default)] class: Option<String>,
    #[props(default)] target: Option<String>,
    #[props(default)] rel: Option<String>,
    #[props(default)] download: Option<String>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let router = try_use_context::<dioxus_router::RouterContext>();
    let origin = try_use_context::<AppOrigin>().and_then(|origin| (origin.0)());
    let current = router.map(|router| router.full_route_string());
    let destination = internal_destination(&href, origin.as_deref(), current.as_deref());
    let native_target = download.is_some()
        || target
            .as_deref()
            .is_some_and(|target| !target.is_empty() && target != "_self");
    let internal = router.is_some() && destination.is_some() && !native_target;
    let destination = destination.unwrap_or_else(|| href.clone());
    // Links with application actions retain their cancellation semantics. Plain
    // links use Dioxus Link directly, including its native modified-click rules.
    if internal && onclick.is_none() {
        return rsx! { Link { to: destination, class, rel, attributes, {children} } };
    }
    rsx! { a { href, class, target, rel, download, onclick: move |event: MouseEvent| {
        if native_target || !event.modifiers().is_empty() || event.trigger_button() != Some(dioxus::html::input_data::MouseButton::Primary) { return; }
        if let Some(handler) = onclick { handler.call(event.clone()); }
        if internal && event.default_action_enabled() && event.modifiers().is_empty()
            && event.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary) {
            event.prevent_default();
            if let Some(router) = router { router.push(destination.clone()); }
        }
    }, ..attributes, {children} } }
}

#[component]
pub fn PageSkeleton() -> Element {
    rsx! {
        section { class: "epsx-page-skeleton p-8 min-h-[55vh] text-inherit [&_i]:block [&_i]:my-4 [&_i]:rounded-lg [&_i]:bg-current [&_i]:opacity-[0.12]", role: "status", aria_busy: "true", aria_label: "Loading page", "data-page-skeleton": "true",
            span { class: "sr-only", "Loading page…" }
            div { aria_hidden: "true", i { class: "w-[35%] h-[1.2rem]" } i { class: "w-[65%] h-[1.2rem]" } i { class: "skeleton-panel h-48" } i { class: "w-[80%] h-[1.2rem]" } }
        }
    }
}

#[derive(Clone, Copy)]
pub struct AdminRevision(pub Signal<u64>);
#[derive(Clone, Copy)]
pub struct AdminAuthenticated(pub Signal<bool>);
#[derive(Clone, Copy)]
pub struct AdminShellOwned;

#[component]
pub fn AdminRouteShell() -> Element {
    let path = use_route::<crate::routes::AdminRoute>().to_string();
    let authenticated = use_context::<AdminAuthenticated>().0;
    rsx! { crate::fullstack::admin::AdminAnalyticsShell {
        authenticated: authenticated(), current_path: path.split(['?', '#']).next().unwrap_or("/").to_owned(),
        SuspenseBoundary { fallback: |_| rsx! { PageSkeleton {} }, AdminRouteContent {} }
    } }
}
#[component]
fn AdminRouteContent() -> Element {
    use_context_provider(|| AdminShellOwned);
    let revision = use_context::<AdminRevision>().0;
    // A keyed list is required to replace the content scope; a key on a lone
    // child does not invalidate its hooks when authentication changes.
    rsx! { for version in [revision()] { Outlet::<crate::routes::AdminRoute> { key: "{version}" } } }
}

/// No navigation interception: preserve scroll and focus after router updates.
pub fn use_navigation_lifecycle() {
    let mut origin = use_signal(|| None::<String>);
    use_context_provider(|| AppOrigin(origin));
    use_effect(move || {
        spawn(async move {
            if let Ok(value) = document::eval(include_str!("navigation_lifecycle.js"))
                .recv::<String>()
                .await
            {
                origin.set(Some(value));
            }
        });
    });
}

#[component]
pub fn QueryForm(
    action: String,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let router = try_use_context::<dioxus_router::RouterContext>();
    let target = action.clone();
    rsx! { form { method: "get", action, class, onsubmit: move |event: FormEvent| {
        if let Some(router) = router {
            event.prevent_default();
            let mut query = url::form_urlencoded::Serializer::new(String::new());
            for (key, value) in event.values() {
                if let dioxus::html::FormValue::Text(value) = value {
                    query.append_pair(&key, &value);
                }
            }
            router.push(format!("{}?{}", target, query.finish()));
        }
    }, ..attributes, {children} } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_targets_preserve_browser_boundaries() {
        for href in [
            "/analytics?page=2&limit=25",
            "/manual",
            "/portfolio/0x123",
            "/#pricing",
        ] {
            assert!(local_route(href));
        }
        for href in [
            "https://pay.epsx.io/checkout/cs_1",
            "//example.com",
            "#main",
            "mailto:help@epsx.io",
            "/api/export",
            "/_server/frontend/logout",
            "/public/logo.svg",
            "/\\evil",
        ] {
            assert!(!local_route(href), "{href}");
        }
    }
    #[test]
    fn same_origin_absolute_and_query_links_preserve_encoded_urls() {
        assert_eq!(
            internal_destination(
                "https://dev.epsx.io/auth?return_url=%2Fanalytics%3Flimit%3D25%26page%3D2#sign",
                Some("https://dev.epsx.io"),
                Some("/")
            ),
            Some("/auth?return_url=%2Fanalytics%3Flimit%3D25%26page%3D2#sign".into())
        );
        assert_eq!(
            internal_destination("?page=2&limit=25", None, Some("/analytics?country=america")),
            Some("/analytics?page=2&limit=25".into())
        );
        for target in [
            "https://pay.epsx.io/checkout/cs_1",
            "https://dev.epsx.io.evil.test/",
            "https://user@dev.epsx.io/",
            "https://dev.epsx.io/api/export",
            "//dev.epsx.io/",
            "#main",
        ] {
            assert!(
                internal_destination(target, Some("https://dev.epsx.io"), Some("/")).is_none(),
                "{target}"
            );
        }
    }
    #[test]
    fn standalone_ssr_keeps_real_links_and_download_attributes() {
        let html = dioxus_ssr::render_element(
            rsx! { AppLink { href: "/analytics?limit=25", "Rankings" } AppLink { href: "/api/export", download: "report.csv", target: "_blank", "Export" } },
        );
        assert!(html.contains("href=\"/analytics?limit=25\""));
        assert!(html.contains("download=\"report.csv\""));
        assert!(html.contains("target=\"_blank\""));
    }
}
