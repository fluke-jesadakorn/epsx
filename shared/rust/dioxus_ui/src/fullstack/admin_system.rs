//! Small route bodies formerly reached only through the legacy dispatcher.
use dioxus::prelude::*;
#[component]
pub fn HydratedAdminHome() -> Element {
    let session = use_server_future(super::admin_auth::auth_session)?;
    if matches!(session.read().as_ref(),Some(Ok(Ok(value)))if value.authenticated) {
        rsx! {super::admin::HydratedAdminDashboard{}}
    } else {
        rsx! {super::admin_auth::HydratedAdminAuth{query:String::new()}}
    }
}
#[component]
pub fn HydratedAdminDenied(query: String, #[props(default)] unauthorized: bool) -> Element {
    crate::pages::news::hydrated::response_status(403);
    let session = use_server_future(super::admin_auth::auth_session)?;
    let authenticated =
        matches!(session.read().as_ref(), Some(Ok(Ok(value))) if value.authenticated);
    let navigator = use_navigator();
    let navigate = use_callback(move |target: String| {
        navigator.push(target);
    });
    let fields = url::form_urlencoded::parse(query.as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect::<std::collections::HashMap<_, _>>();
    let mut reason = if unauthorized {
        "You are not authorized to view this resource.".into()
    } else {
        fields
            .get("reason")
            .cloned()
            .unwrap_or_else(|| "Access denied".into())
    };
    if !unauthorized {
        if let Some(detail) = fields.get("detail") {
            reason.push_str(": ");
            reason.push_str(detail);
        }
        for key in ["permission", "context", "route"] {
            if let Some(value) = fields.get(key) {
                reason.push_str(&format!(" [{key}={value}]"));
            }
        }
    }
    rsx! {
        document::Title{"Access denied | EPSX Admin"}
        document::Meta{name:"robots",content:"noindex"}
        document::Link{rel:"stylesheet",href:"/public/dist/tailwind.css"}
        document::Link{rel:"stylesheet",href:"/_ui/admin.css"}
        super::admin::AdminAnalyticsShell { authenticated, current_path: if unauthorized { "/unauthorized" } else { "/access-denied" }, title: "Access denied",
          div{class:"grid place-items-center p-6",
            crate::auth::AccessDenied{reason:Some(reason),required_permissions:unauthorized.then(||vec!["admin:*".into()]),on_navigate:Some(navigate)}
          }
        }
    }
}
#[component]
pub fn HydratedAdminNotFound() -> Element {
    let session = use_server_future(super::admin_auth::auth_session)?;
    let authenticated =
        matches!(session.read().as_ref(), Some(Ok(Ok(value))) if value.authenticated);
    crate::pages::news::hydrated::response_status(404);
    rsx! {
        document::Title{"Page not found | EPSX Admin"}
        document::Meta{name:"robots",content:"noindex"}
        document::Link{rel:"stylesheet",href:"/public/dist/tailwind.css"}
        document::Link{rel:"stylesheet",href:"/_ui/admin.css"}
        super::admin::AdminAnalyticsShell { authenticated, current_path: "/404", title: "Page not found",
          div{class:"grid place-items-center p-6",
            section{class:"max-w-lg text-center space-y-5",div{class:"text-7xl font-bold text-primary","404"}h1{class:"text-3xl font-semibold","Page not found"}p{class:"text-muted-foreground","The page you are looking for does not exist."}Link{class:"btn btn-primary",to:"/","Back to home"}}
          }
        }
    }
}
