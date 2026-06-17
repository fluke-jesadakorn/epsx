//! NavbarSkeleton — port of `apps-old/frontend/components/nav/navbar-skeleton.tsx`.
//!
//! Pure skeleton state for the header. Renders a sticky header with three
//! pulsing rectangles (logo placeholder + 2 desktop buttons + 1 mobile
//! hamburger). Uses Tailwind's `animate-pulse` utility.

use dioxus::prelude::*;

/// Loading skeleton for the navbar. Renders the same sticky
/// `epsx-header` chrome with `bg-slate-100` / `dark:bg-slate-800`
/// `animate-pulse` placeholders.
#[component]
pub fn NavbarSkeleton() -> Element {
    rsx! {
        header {
            class: "epsx-header sticky top-0 z-50",
            div { class: "mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6",
                // Logo placeholder
                div { class: "h-6 w-14 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" }
                // Right cluster
                div { class: "flex items-center gap-2",
                    div { class: "hidden md:flex items-center gap-2",
                        div { class: "h-7 w-16 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" }
                        div { class: "h-7 w-20 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" }
                    }
                    div { class: "h-9 w-9 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse lg:hidden" }
                }
            }
        }
    }
}
