//! Root application components for EPSX Frontend and Admin Fullstack apps.

use crate::pages::PageContext;
use crate::routes::{AdminRoute, FrontendRoute};
use dioxus::prelude::*;

#[component]
pub fn FrontendApp(#[props(default = None)] initial_context: Option<PageContext>) -> Element {
    if let Some(ctx) = initial_context {
        use_context_provider(|| ctx);
    }
    rsx! {
        document::Title { "EPSX — Analytics Platform" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        Router::<FrontendRoute> {}
    }
}

#[component]
pub fn AdminApp(#[props(default = None)] initial_context: Option<PageContext>) -> Element {
    if let Some(ctx) = initial_context {
        use_context_provider(|| ctx);
    }
    rsx! {
        document::Title { "EPSX Admin — Management Portal" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        Router::<AdminRoute> {}
    }
}
