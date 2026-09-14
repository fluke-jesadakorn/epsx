//! Root application components for EPSX Frontend and Admin Fullstack apps.

use crate::pages::PageContext;
use crate::routes::{AdminRoute, FrontendRoute};
use dioxus::prelude::*;

/// Frontend styles are supplied by its external Tailwind bundle.
#[derive(Clone, Copy)]
pub struct FrontendTailwindStyles;

#[component]
pub fn FrontendApp(#[props(default = None)] initial_context: Option<PageContext>) -> Element {
    if let Some(ctx) = initial_context {
        use_context_provider(|| ctx);
    }
    use_context_provider(|| FrontendTailwindStyles);
    crate::navigation::use_navigation_lifecycle();
    rsx! {
        document::Link { rel: "stylesheet", href: "/public/dist/tailwind.css" }
        document::Title { "EPSX — Analytics Platform" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        Router::<FrontendRoute> {}
    }
}

#[component]
pub fn AdminApp(#[props(default = None)] initial_context: Option<PageContext>) -> Element {
    crate::navigation::use_navigation_lifecycle();
    let session = use_server_future(crate::fullstack::admin_auth::auth_session)?;
    let mut attempted = use_signal(|| false);
    let mut revision = use_signal(|| 0u64);
    use_context_provider(|| crate::navigation::AdminRevision(revision));
    let mut authenticated = use_signal(
        || matches!(session.read().as_ref(), Some(Ok(Ok(value))) if value.authenticated),
    );
    use_context_provider(|| crate::navigation::AdminAuthenticated(authenticated));
    use_context_provider(|| crate::fullstack::admin_auth::AdminRecoveryOwned);
    use_effect(move || {
        #[cfg(feature = "web")]
        {
            let recover = matches!(session.read().as_ref(),Some(Ok(Ok(value))) if value.recover_session&&!value.authenticated&&!value.verifier_unavailable);
            if recover && !*attempted.peek() {
                attempted.set(true);
                spawn(async move {
                    if matches!(crate::fullstack::admin_auth::auth_action(crate::fullstack::admin_auth::AuthCommand::Refresh).await,Ok(Ok(reply)) if reply.authenticated)
                    {
                        authenticated.set(true);
                        let next = *revision.peek() + 1;
                        revision.set(next);
                    }
                });
            }
        }
        #[cfg(not(feature = "web"))]
        {
            let _ = (&session, &attempted, &revision, &authenticated);
        }
    });
    let theme = use_signal(|| true);
    use_context_provider(|| crate::fullstack::admin::AdminTheme(theme));
    if let Some(ctx) = initial_context {
        use_context_provider(|| ctx);
    }
    rsx! {
        document::Title { "EPSX Admin — Management Portal" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        Router::<AdminRoute> {}
    }
}

#[component]
pub fn FrontendRoot() -> Element {
    rsx! { FrontendApp {} }
}

#[component]
pub fn AdminRoot() -> Element {
    rsx! { AdminApp {} }
}
