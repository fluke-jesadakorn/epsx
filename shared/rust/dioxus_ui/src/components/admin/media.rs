//! Admin media sub-components — 1:1 mirror of the Next.js source's
//! `apps-old/admin-frontend/components/media/media-browser.tsx` (337 LoC).
//!
//! Section markers (mirrored 1:1 from `pages/admin_pages/media.rs`):
//!   - `media-browser`   → `MediaBrowser`
//!   - `media-uploader`  → `MediaUploader`
//!   - `media-filters`   → `MediaFilters`
//!   - `media-stats`     → `MediaStats`
//!
//! Wave 6C Track C — extracted from the Wave 6B Track A port of the
//! media page.

use crate::primitives::*;

use dioxus::prelude::*;

// ============================================================================
// MediaStats — top-of-page stat row (total files, total size, newest,
// oldest). Mirrors the TS source's "media-stats" surface.
// ============================================================================

#[component]
pub fn MediaStats() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c media-stats ===
        div { class: "media-stats grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6",
            StatCard { label: "Total files".to_string(), value: "1,247".to_string(), icon: Some("file-text".to_string()) }
            StatCard { label: "Total size".to_string(), value: "847 MB".to_string(), icon: Some("database".to_string()) }
            StatCard { label: "Newest upload".to_string(), value: "2 min ago".to_string(), icon: Some("clock".to_string()) }
            StatCard { label: "Oldest upload".to_string(), value: "2024-01-15".to_string(), icon: Some("calendar".to_string()) }
        }
    }
}

// ============================================================================
// MediaUploader — upload dropzone + button.
// Mirrors the TS source's hidden file input + Upload button pattern.
// ============================================================================

#[component]
pub fn MediaUploader() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c media-uploader ===
        div { class: "card card-glass media-uploader mb-4",
            div { class: "card-body",
                FileUpload {
                    name: "media".to_string(),
                    label: Some("Drop files here or click to upload".to_string()),
                    multiple: true,
                    accept: Some("image/*,video/*,.pdf".to_string()),
                    help: Some("Max 50 MB per file".to_string()),
                }
            }
        }
    }
}

// ============================================================================
// MediaFilters — bucket tabs (news / chat / notifications / public) +
// type filter (image / video / doc) + sort.
// Mirrors the TS source's bucket tabs + search bar + view toggle.
// ============================================================================

#[component]
pub fn MediaFilters(bucket: String, view: String) -> Element {
    let mut bucket_signal = use_signal(|| bucket);
    let mut view_signal = use_signal(|| view);
    rsx! {
        // === wave6b-admin-pages-depth-track-c media-filters ===
        div { class: "card card-glass media-filters mb-4",
            div { class: "card-body",
                div { class: "flex flex-col md:flex-row gap-3 items-stretch md:items-center",
                    // Bucket tabs (pill buttons).
                    div { class: "flex flex-wrap gap-1.5",
                        for b in ["news", "chat", "notifications", "public"].iter() {
                            button {
                                class: if *bucket_signal.read() == *b { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                                r#type: "button",
                                onclick: move |_| bucket_signal.set(b.to_string()),
                                {b.to_string()}
                            }
                        }
                    }
                    // Type filter.
                    div { class: "md:w-32 ml-auto",
                        select { class: "input",
                            option { value: "all", "All types" }
                            option { value: "image", "Images" }
                            option { value: "video", "Videos" }
                            option { value: "doc", "Documents" }
                        }
                    }
                    // Search.
                    div { class: "md:w-48",
                        input { class: "input", r#type: "text", placeholder: "Filter files…" }
                    }
                    // View toggle.
                    div { class: "flex items-end gap-1",
                        button {
                            class: if *view_signal.read() == "grid" { "btn btn-icon btn-primary" } else { "btn btn-icon btn-ghost" },
                            r#type: "button",
                            "aria-label": "Grid view",
                            Icon { name: "image".to_string(), size: Some(16) }
                        }
                        button {
                            class: if *view_signal.read() == "list" { "btn btn-icon btn-primary" } else { "btn btn-icon btn-ghost" },
                            r#type: "button",
                            "aria-label": "List view",
                            Icon { name: "file-text".to_string(), size: Some(16) }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// MediaBrowser — the file grid + list view. Mirrors the TS source's
// `MediaBrowser` component body (lines 250-336).
// ============================================================================

#[component]
pub fn MediaBrowser(bucket: String, view: String) -> Element {
    let files: Vec<(String, String, String)> = vec![
        ("news_2024-09-20_banner.png".to_string(), "234 KB".to_string(), "image".to_string()),
        ("news_2024-09-19_hero.jpg".to_string(), "1.2 MB".to_string(), "image".to_string()),
        ("news_2024-09-18_chart.png".to_string(), "412 KB".to_string(), "image".to_string()),
        ("chat_avatar_001.png".to_string(), "12 KB".to_string(), "image".to_string()),
        ("chat_avatar_002.png".to_string(), "14 KB".to_string(), "image".to_string()),
        ("chat_attachment_2024-09-15.pdf".to_string(), "847 KB".to_string(), "doc".to_string()),
        ("notification_banner.png".to_string(), "78 KB".to_string(), "image".to_string()),
        ("notification_icon.png".to_string(), "4 KB".to_string(), "image".to_string()),
        ("public_og_default.png".to_string(), "92 KB".to_string(), "image".to_string()),
        ("public_favicon.png".to_string(), "2 KB".to_string(), "image".to_string()),
        ("public_whitepaper.pdf".to_string(), "2.1 MB".to_string(), "doc".to_string()),
        ("public_terms.pdf".to_string(), "184 KB".to_string(), "doc".to_string()),
    ];
    rsx! {
        // === wave6b-admin-pages-depth-track-c media-browser ===
        div { class: "media-browser",
            // Bucket label.
            div { class: "flex items-center justify-between mb-3",
                h3 { class: "text-sm font-bold text-muted-foreground uppercase tracking-widest",
                    "Bucket: {bucket}"
                }
                span { class: "text-xs text-muted-foreground font-mono",
                    "{files.len()} files"
                }
            }
            if view == "grid" {
                // Grid view.
                div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4",
                    for (name, size, kind) in files.iter() {
                        MediaGridItem { name: name.clone(), size: size.clone(), kind: kind.clone() }
                    }
                }
            } else {
                // List view.
                div { class: "space-y-2",
                    for (name, size, kind) in files.iter() {
                        MediaListItem { name: name.clone(), size: size.clone(), kind: kind.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn MediaGridItem(name: String, size: String, kind: String) -> Element {
    let icon = if kind == "image" { "image" } else { "file-text" };
    rsx! {
        div { class: "card card-glass hover-scale group",
            div { class: "card-body p-3",
                div { class: "aspect-square bg-muted/30 rounded mb-2 flex items-center justify-center text-muted-foreground",
                    Icon { name: icon.to_string(), size: Some(32) }
                }
                p { class: "text-xs truncate", "{name}" }
                div { class: "flex items-center justify-between mt-1",
                    p { class: "text-xs text-muted-foreground font-mono", "{size}" }
                    div { class: "opacity-0 group-hover:opacity-100 flex gap-1",
                        a { class: "btn btn-sm btn-icon btn-ghost", href: "#", "aria-label": "Open", Icon { name: "external-link".to_string(), size: Some(12) } }
                        a { class: "btn btn-sm btn-icon btn-ghost", href: "#", "aria-label": "Copy URL", Icon { name: "link".to_string(), size: Some(12) } }
                        a { class: "btn btn-sm btn-icon btn-ghost text-danger", href: "#", "aria-label": "Delete", Icon { name: "x".to_string(), size: Some(12) } }
                    }
                }
            }
        }
    }
}

#[component]
fn MediaListItem(name: String, size: String, kind: String) -> Element {
    let icon = if kind == "image" { "image" } else { "file-text" };
    rsx! {
        div { class: "flex items-center gap-4 p-3 rounded-xl bg-card border border-border/40 hover:border-border group",
            div { class: "shrink-0 w-10 h-10 rounded-lg overflow-hidden bg-muted/30 border border-border/40 flex items-center justify-center text-muted-foreground",
                Icon { name: icon.to_string(), size: Some(16) }
            }
            div { class: "flex-1 min-w-0",
                p { class: "text-sm font-medium truncate", "{name}" }
                p { class: "text-xs text-muted-foreground font-mono truncate", "{kind}" }
            }
            span { class: "text-xs text-muted-foreground whitespace-nowrap hidden sm:block", "{size}" }
            div { class: "flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100",
                a { class: "btn btn-sm btn-icon btn-ghost", href: "#", "aria-label": "Open", Icon { name: "external-link".to_string(), size: Some(14) } }
                a { class: "btn btn-sm btn-icon btn-ghost", href: "#", "aria-label": "Copy URL", Icon { name: "link".to_string(), size: Some(14) } }
                a { class: "btn btn-sm btn-icon btn-ghost text-danger", href: "#", "aria-label": "Delete", Icon { name: "x".to_string(), size: Some(14) } }
            }
        }
    }
}

// ============================================================================
// Tests — Wave 6C Track C per-sub-component smoke tests.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: dioxus::prelude::Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// `MediaStats` renders the 4 stat cards.
    #[test]
    fn media_stats_renders() {
        let el = rsx! { MediaStats {} };
        let html = render_to_string(el);
        assert!(html.contains("media-stats"), "MediaStats must emit its section marker. Got: {html}");
        assert!(html.contains("Total files"), "MediaStats must render the stat label. Got: {html}");
    }

    /// `MediaUploader` renders the dropzone card.
    #[test]
    fn media_uploader_renders() {
        let el = rsx! { MediaUploader {} };
        let html = render_to_string(el);
        assert!(html.contains("media-uploader"), "MediaUploader must emit its section marker. Got: {html}");
    }

    /// `MediaFilters` renders the bucket tabs + type filter + view toggle.
    #[test]
    fn media_filters_renders() {
        let el = rsx! { MediaFilters { bucket: "news".to_string(), view: "grid".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("media-filters"), "MediaFilters must emit its section marker. Got: {html}");
    }

    /// `MediaBrowser` renders the file grid (when view == "grid").
    #[test]
    fn media_browser_renders_grid() {
        let el = rsx! { MediaBrowser { bucket: "news".to_string(), view: "grid".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("media-browser"), "MediaBrowser must emit its section marker. Got: {html}");
        assert!(html.contains("Bucket:"), "MediaBrowser must render the bucket label. Got: {html}");
    }

    /// `MediaBrowser` renders the file list (when view == "list").
    #[test]
    fn media_browser_renders_list() {
        let el = rsx! { MediaBrowser { bucket: "chat".to_string(), view: "list".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("media-browser"), "MediaBrowser must emit its section marker. Got: {html}");
    }
}
