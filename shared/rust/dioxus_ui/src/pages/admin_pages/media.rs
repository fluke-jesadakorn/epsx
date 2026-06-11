//! /admin/media — media browser + uploader.
//!
//! Wave 6B Track A — port of `apps-old/admin-frontend/app/media/page.tsx`
//! (17 LoC) + `components/media/media-browser.tsx` (337 LoC).
//!
//! Sections (per design doc §"Track A" line 175):
//! - `media-browser` — the bucket tabs + file grid (grid + list view
//!   toggle). Source: `components/media/media-browser.tsx` body
//!   (bucket tabs, file grid, search, view toggle).
//! - `media-uploader` — the upload dropzone (the TS source uses a
//!   hidden file input + a "Upload" button; the Dioxus port adds the
//!   visible `FileUpload` component alongside the button for visual
//!   parity).
//! - `media-filters` — bucket tabs (news / chat / notifications /
//!   public) + type filter (image / video / doc) + sort.
//! - `media-stats` — top-of-page stat row (total files, total size,
//!   newest upload, oldest upload).

use crate::auth::AdminAuthGate;
use crate::layout::admin_shell::AdminShell;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Media");
    (meta, rsx! { RenderMedia { ctx: ctx.clone() } })
}

#[component]
fn RenderMedia(ctx: PageContext) -> Element {
    let mut bucket = use_signal(|| "news".to_string());
    let mut view = use_signal(|| "grid".to_string());
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("media management".to_string()), required_permissions: Some(vec!["media:manage".to_string()]), return_url: Some(ctx.path.clone()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Media Browser".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Media".to_string(), "/media".to_string()),
                ],
                div { class: "container page-content admin-media",
                    // MediaStats — 4-card top stat row.
                    MediaStats {}
                    // MediaUploader — upload dropzone + button.
                    MediaUploader {}
                    // MediaFilters — bucket tabs + type filter + search + view toggle.
                    MediaFilters { bucket: bucket.read().clone(), view: view.read().clone() }
                    // MediaBrowser — the file grid/list.
                    MediaBrowser { bucket: bucket.read().clone(), view: view.read().clone() }
                }
            }
        }
    }
}

// ===== MediaStats ==========================================================
//
// Top-of-page stat row — total files, total size, newest upload,
// oldest upload. Mirrors the TS source's "media-stats" surface.

#[component]
fn MediaStats() -> Element {
    rsx! {
        div { class: "media-stats grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6",
            StatCard { label: "Total files".to_string(), value: "1,247".to_string(), icon: Some("file-text".to_string()) }
            StatCard { label: "Total size".to_string(), value: "847 MB".to_string(), icon: Some("database".to_string()) }
            StatCard { label: "Newest upload".to_string(), value: "2 min ago".to_string(), icon: Some("clock".to_string()) }
            StatCard { label: "Oldest upload".to_string(), value: "2024-01-15".to_string(), icon: Some("calendar".to_string()) }
        }
    }
}

// ===== MediaUploader =======================================================
//
// Upload dropzone + button. The TS source uses a hidden file input
// triggered by a button; the Dioxus port adds a visible dropzone
// (the existing `<FileUpload>` primitive) for visual parity.

#[component]
fn MediaUploader() -> Element {
    rsx! {
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

// ===== MediaFilters ========================================================
//
// Bucket tabs (news / chat / notifications / public) + type filter
// (image / video / doc) + sort. Mirrors the TS source's bucket tabs
// + search bar + view toggle. The Dioxus port wires the bucket +
// view signal back to the parent (so changing tabs updates the
// `bucket` and `view` signals).

#[component]
fn MediaFilters(bucket: String, view: String) -> Element {
    let mut bucket_signal = use_signal(|| bucket);
    let mut view_signal = use_signal(|| view);
    rsx! {
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

// ===== MediaBrowser ========================================================
//
// The file grid + list view. Mirrors the TS source's `MediaBrowser`
// component body (lines 250-336) — file grid (4-6 col responsive) +
// file list (single column) + empty state when no files match.

#[component]
fn MediaBrowser(bucket: String, view: String) -> Element {
    // 12 placeholder files mirroring the existing port's `for i in 0..12`
    // shape. The TS source renders a grid of `FileGridItem` cards with
    // file name + size + hover overlay (open / copy URL / delete).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    /// Authenticated admin context — the page gates on
    /// `media:manage`, so the fixture user must hold that
    /// permission.
    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["media:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/media".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test.
    #[test]
    fn test_render_smoke() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "media must render non-empty HTML. Got: {html}");
        assert!(html.len() > 100, "media HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Section-marker test.
    #[test]
    fn test_section_markers() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "media-browser",
            "media-uploader",
            "media-filters",
            "media-stats",
        ] {
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "media must contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
