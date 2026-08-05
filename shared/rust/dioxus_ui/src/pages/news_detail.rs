//! Public news detail backed by the content service's slug-scoped outcome.
//!
//! Unknown content is a real not-found state. Dependency and envelope failures
//! are explicit retryable errors; neither path synthesizes an article.

use super::{PageContext, PageMeta, PageStatus};
use crate::layout::main_layout::MainLayout;
use crate::primitives::*;
use chrono::NaiveDate;
use dioxus::prelude::*;
use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use url::Url;

const MAX_BODY_BYTES: usize = 256 * 1_024;
const MAX_MARKDOWN_EVENTS: usize = 32_000;
const MAX_MARKDOWN_NESTING: usize = 32;
const MAX_TABLE_COLUMNS: usize = 20;
const MAX_TABLE_ROWS: usize = 500;
const MAX_CODE_BLOCK_BYTES: usize = 64 * 1_024;
const MAX_LINK_TITLE_CHARS: usize = 256;

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
struct NewsArticle {
    id: Option<String>,
    slug: String,
    title: String,
    #[serde(default)]
    summary: Option<String>,
    body: String,
    cover_image_url: Option<String>,
    author: Option<String>,
    published_at: Option<String>,
    tags: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
enum NewsDetailOutcome {
    Ready { article: NewsArticle },
    NotFound,
    Error { code: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SafeLinkTarget {
    Internal(String),
    External(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SafeTableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SafeMarkdownNode {
    Text(String),
    Paragraph(Vec<SafeMarkdownNode>),
    Heading {
        level: u8,
        children: Vec<SafeMarkdownNode>,
    },
    SoftBreak,
    HardBreak,
    Emphasis(Vec<SafeMarkdownNode>),
    Strong(Vec<SafeMarkdownNode>),
    Strikethrough(Vec<SafeMarkdownNode>),
    InlineCode(String),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    BlockQuote(Vec<SafeMarkdownNode>),
    HorizontalRule,
    List {
        start: Option<u64>,
        children: Vec<SafeMarkdownNode>,
    },
    ListItem(Vec<SafeMarkdownNode>),
    TaskMarker(bool),
    Link {
        target: SafeLinkTarget,
        title: Option<String>,
        children: Vec<SafeMarkdownNode>,
    },
    Table(Vec<SafeMarkdownNode>),
    TableHead(Vec<SafeMarkdownNode>),
    TableRow(Vec<SafeMarkdownNode>),
    TableCell {
        alignment: SafeTableAlignment,
        header: bool,
        children: Vec<SafeMarkdownNode>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SafeMarkdownContent {
    Rich {
        nodes: Vec<SafeMarkdownNode>,
        visible_text: String,
    },
    Plaintext(String),
}

impl SafeMarkdownContent {
    fn visible_text(&self) -> &str {
        match self {
            Self::Rich { visible_text, .. } | Self::Plaintext(visible_text) => visible_text,
        }
    }
}

#[derive(Debug)]
enum MarkdownFrameKind {
    Root,
    Paragraph,
    Heading(u8),
    BlockQuote,
    List(Option<u64>),
    ListItem,
    Emphasis,
    Strong,
    Strikethrough,
    Link {
        target: Option<SafeLinkTarget>,
        title: Option<String>,
    },
    Image,
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Table {
        alignments: Vec<SafeTableAlignment>,
        rows: usize,
    },
    TableHead {
        cells: usize,
    },
    TableRow {
        cells: usize,
    },
    TableCell {
        alignment: SafeTableAlignment,
        header: bool,
    },
    Transparent,
}

#[derive(Debug)]
struct MarkdownFrame {
    end: Option<TagEnd>,
    kind: MarkdownFrameKind,
    children: Vec<SafeMarkdownNode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MarkdownLimit;

fn safe_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn safe_text(value: &str, max: usize) -> bool {
    value.chars().count() <= max && !value.chars().any(char::is_control)
}

fn safe_cover(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 2_048
        || value.chars().any(char::is_control)
        || value.contains('\\')
        || value.contains('#')
    {
        return false;
    }
    if value.starts_with('/') {
        return !value.starts_with("//")
            && Url::parse("https://epsx.invalid/")
                .and_then(|base| base.join(value))
                .is_ok_and(|url| {
                    url.scheme() == "https"
                        && url.host_str() == Some("epsx.invalid")
                        && url.username().is_empty()
                        && url.password().is_none()
                        && url.fragment().is_none()
                });
    }
    let Some(authority) = canonical_https_authority(value) else {
        return false;
    };
    if authority.contains('@') {
        return false;
    }
    Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.fragment().is_none()
    })
}

fn valid_display_date(value: &str) -> bool {
    let Some((_, year)) = value.rsplit_once(", ") else {
        return false;
    };
    year.len() == 4
        && year.bytes().all(|byte| byte.is_ascii_digit())
        && year != "0000"
        && NaiveDate::parse_from_str(value, "%B %e, %Y").is_ok()
}

fn valid_article(article: &NewsArticle, expected_slug: &str) -> bool {
    safe_slug(&article.slug)
        && article.slug == expected_slug
        && article.id.as_deref().is_none_or(|id| safe_text(id, 128))
        && !article.title.trim().is_empty()
        && article.title == article.title.trim()
        && safe_text(&article.title, 200)
        && article
            .summary
            .as_deref()
            .is_none_or(|summary| safe_text(summary, 500))
        && !article.body.trim().is_empty()
        && article.body.len() <= MAX_BODY_BYTES
        && article.tags.len() <= 32
        && article
            .tags
            .iter()
            .all(|tag| !tag.trim().is_empty() && safe_text(tag, 64))
        && article.cover_image_url.as_deref().is_none_or(safe_cover)
        && article.author.as_deref().is_none_or(|author| {
            !author.trim().is_empty() && author == author.trim() && safe_text(author, 120)
        })
        && article
            .published_at
            .as_deref()
            .is_none_or(valid_display_date)
}

fn parse_outcome(ctx: &PageContext, slug: &str) -> NewsDetailOutcome {
    if !safe_slug(slug) {
        return NewsDetailOutcome::NotFound;
    }
    let Some(raw) = ctx.params.get("data_news_post") else {
        return NewsDetailOutcome::Error {
            code: "missing_content_outcome".to_string(),
        };
    };
    let Ok(outcome) = serde_json::from_str::<NewsDetailOutcome>(raw) else {
        return NewsDetailOutcome::Error {
            code: "malformed_content_response".to_string(),
        };
    };
    match outcome {
        NewsDetailOutcome::Ready { article } if valid_article(&article, slug) => {
            NewsDetailOutcome::Ready { article }
        }
        NewsDetailOutcome::NotFound => NewsDetailOutcome::NotFound,
        NewsDetailOutcome::Error { code } if !code.is_empty() && safe_text(&code, 64) => {
            NewsDetailOutcome::Error { code }
        }
        _ => NewsDetailOutcome::Error {
            code: "malformed_content_response".to_string(),
        },
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let slug = ctx.params.get("slug").cloned().unwrap_or_default();
    let outcome = parse_outcome(ctx, &slug);
    let mut meta = PageMeta::marketing("News article");
    match &outcome {
        NewsDetailOutcome::Ready { article } => {
            meta.title = format!("{} — EPSX News", article.title);
            meta.description = article
                .summary
                .clone()
                .filter(|summary| !summary.trim().is_empty())
                .unwrap_or_else(|| article.title.clone());
        }
        NewsDetailOutcome::NotFound => {
            meta.title = "Article Not Found — EPSX".to_string();
            meta.description = "The requested published news article was not found.".to_string();
            meta.status = PageStatus::NotFound;
        }
        NewsDetailOutcome::Error { .. } => {
            meta.title = "News unavailable — EPSX".to_string();
            meta.description = "The requested news article could not be loaded.".to_string();
        }
    }
    let retry_href = if ctx.query.is_empty() {
        ctx.path.clone()
    } else {
        format!("{}?{}", ctx.path, ctx.query)
    };

    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                match outcome {
                    NewsDetailOutcome::Ready { article } => rsx! { NewsArticleView { article } },
                    NewsDetailOutcome::NotFound => rsx! { NewsNotFound {} },
                    NewsDetailOutcome::Error { .. } => rsx! { NewsDetailError { retry_href } },
                }
            }
        },
    )
}

#[component]
fn NewsArticleView(article: NewsArticle) -> Element {
    let content = parse_safe_markdown(&article.body);
    let read_time = format!("{} min", read_minutes(content.visible_text()));
    rsx! {
        article { class: "news-detail-body",
            section { class: "relative w-full overflow-hidden isolate news-detail-hero",
                if let Some(cover) = &article.cover_image_url {
                    img { class: "absolute inset-0 w-full h-full object-cover", src: cover, alt: "" }
                    div { class: "absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-black/20" }
                } else {
                    div { class: "absolute inset-0 bg-gradient-to-br from-cyan-500/8 via-background to-purple-500/8" }
                }
                div { class: "relative z-10 max-w-4xl mx-auto px-4 sm:px-6 pt-8 pb-12 flex flex-col min-h-[240px] sm:min-h-[300px]",
                    a { class: "inline-flex items-center gap-2 text-sm mb-auto transition-colors news-detail-back", href: "/news",
                        Icon { name: "arrow-left".to_string(), size: Some(16) }
                        " Back to News"
                    }
                    div {
                        if !article.tags.is_empty() {
                            div { class: "flex flex-wrap gap-2 mb-5",
                                for tag in article.tags.iter() {
                                    span { class: "px-3 py-1 rounded-full text-[11px] font-bold tracking-[0.15em] uppercase bg-cyan-500/15 text-cyan-500 border border-cyan-500/25", "{tag}" }
                                }
                            }
                        }
                        h1 { class: "text-3xl sm:text-4xl lg:text-[2.75rem] font-extrabold leading-[1.1] tracking-tight mb-5", "{article.title}" }
                        div { class: "flex flex-wrap items-center gap-5 text-sm text-muted-foreground",
                            if let Some(date) = &article.published_at {
                                span { class: "flex items-center gap-1.5", Icon { name: "calendar".to_string(), size: Some(14) } " {date}" }
                            }
                            span { class: "flex items-center gap-1.5", Icon { name: "clock".to_string(), size: Some(14) } " {read_time} read" }
                            if let Some(author) = &article.author {
                                span { class: "flex items-center gap-1.5", Icon { name: "user".to_string(), size: Some(14) } " {author}" }
                            }
                        }
                    }
                }
            }
            div { class: "h-[3px] news-detail-accent bg-gradient-to-r from-cyan-500 via-purple-500 to-cyan-500" }
            div { class: "max-w-3xl mx-auto px-4 sm:px-6 pt-12 pb-20 news-detail-content",
                SafeMarkdownView { content }
                div { class: "mt-16 pt-8 border-t border-border/20 news-detail-footer",
                    a { class: "inline-flex items-center gap-3 px-5 py-3 rounded-xl text-sm font-medium text-muted-foreground hover:text-foreground bg-card/50 hover:bg-card border border-border/20 hover:border-border/40 transition-all group news-detail-back-link", href: "/news",
                        Icon { name: "arrow-left".to_string(), size: Some(16) }
                        " Back to all articles"
                    }
                }
            }
        }
    }
}

#[component]
fn SafeMarkdownView(content: SafeMarkdownContent) -> Element {
    match content {
        SafeMarkdownContent::Rich { nodes, .. } => rsx! {
            div { class: "prose prose-lg prose-neutral dark:prose-invert max-w-none news-markdown",
                SafeMarkdownNodes { nodes }
            }
        },
        SafeMarkdownContent::Plaintext(text) => rsx! {
            pre {
                class: "prose prose-lg prose-neutral dark:prose-invert max-w-none whitespace-pre-wrap news-markdown-fallback",
                "data-news-markdown-fallback": "true",
                "{text}"
            }
        },
    }
}

#[component]
fn SafeMarkdownNodes(nodes: Vec<SafeMarkdownNode>) -> Element {
    rsx! {
        for node in nodes.iter() {
            match node {
                SafeMarkdownNode::Text(value) => rsx! { "{value}" },
                SafeMarkdownNode::Paragraph(children) => rsx! {
                    p { class: "news-markdown-paragraph",
                        SafeMarkdownNodes { nodes: children.clone() }
                    }
                },
                SafeMarkdownNode::Heading { level, children } => match level {
                    2 => rsx! { h2 { class: "news-markdown-heading-2", SafeMarkdownNodes { nodes: children.clone() } } },
                    3 => rsx! { h3 { class: "news-markdown-heading-3", SafeMarkdownNodes { nodes: children.clone() } } },
                    4 => rsx! { h4 { class: "news-markdown-heading-4", SafeMarkdownNodes { nodes: children.clone() } } },
                    5 => rsx! { h5 { class: "news-markdown-heading-5", SafeMarkdownNodes { nodes: children.clone() } } },
                    _ => rsx! { h6 { class: "news-markdown-heading-6", SafeMarkdownNodes { nodes: children.clone() } } },
                },
                SafeMarkdownNode::SoftBreak => rsx! { "\n" },
                SafeMarkdownNode::HardBreak => rsx! { br {} },
                SafeMarkdownNode::Emphasis(children) => rsx! {
                    em { class: "news-markdown-emphasis", SafeMarkdownNodes { nodes: children.clone() } }
                },
                SafeMarkdownNode::Strong(children) => rsx! {
                    strong { class: "news-markdown-strong", SafeMarkdownNodes { nodes: children.clone() } }
                },
                SafeMarkdownNode::Strikethrough(children) => rsx! {
                    del { class: "news-markdown-strikethrough", SafeMarkdownNodes { nodes: children.clone() } }
                },
                SafeMarkdownNode::InlineCode(value) => rsx! {
                    code { class: "news-markdown-inline-code", "{value}" }
                },
                SafeMarkdownNode::CodeBlock { language, code } => rsx! {
                    div { class: "news-markdown-code-block",
                        if let Some(language) = language {
                            span { class: "news-markdown-code-language", "{language}" }
                        }
                        pre { class: "news-markdown-pre",
                            code { class: "news-markdown-code", "{code}" }
                        }
                    }
                },
                SafeMarkdownNode::BlockQuote(children) => rsx! {
                    blockquote { class: "news-markdown-blockquote",
                        SafeMarkdownNodes { nodes: children.clone() }
                    }
                },
                SafeMarkdownNode::HorizontalRule => rsx! {
                    hr { class: "news-markdown-rule" }
                },
                SafeMarkdownNode::List { start, children } => if let Some(start) = start {
                    rsx! {
                        ol { class: "news-markdown-ordered-list", start: "{start}",
                            SafeMarkdownNodes { nodes: children.clone() }
                        }
                    }
                } else {
                    rsx! {
                        ul { class: "news-markdown-unordered-list",
                            SafeMarkdownNodes { nodes: children.clone() }
                        }
                    }
                },
                SafeMarkdownNode::ListItem(children) => rsx! {
                    li { class: "news-markdown-list-item",
                        SafeMarkdownNodes { nodes: children.clone() }
                    }
                },
                SafeMarkdownNode::TaskMarker(checked) => rsx! {
                    input {
                        class: "news-markdown-task-marker",
                        r#type: "checkbox",
                        disabled: true,
                        checked: *checked,
                        aria_label: if *checked { "Completed task" } else { "Incomplete task" },
                    }
                },
                SafeMarkdownNode::Link { target, title, children } => match target {
                    SafeLinkTarget::Internal(href) => if let Some(title) = title {
                        rsx! {
                            a { class: "news-markdown-link", href: href, title: title,
                                SafeMarkdownNodes { nodes: children.clone() }
                            }
                        }
                    } else {
                        rsx! {
                            a { class: "news-markdown-link", href: href,
                                SafeMarkdownNodes { nodes: children.clone() }
                            }
                        }
                    },
                    SafeLinkTarget::External(href) => if let Some(title) = title {
                        rsx! {
                            a {
                                class: "news-markdown-link",
                                href: href,
                                title: title,
                                rel: "nofollow noopener noreferrer",
                                SafeMarkdownNodes { nodes: children.clone() }
                            }
                        }
                    } else {
                        rsx! {
                            a {
                                class: "news-markdown-link",
                                href: href,
                                rel: "nofollow noopener noreferrer",
                                SafeMarkdownNodes { nodes: children.clone() }
                            }
                        }
                    },
                },
                SafeMarkdownNode::Table(children) => rsx! {
                    div { class: "max-w-full overflow-x-auto news-markdown-table-scroll",
                        table { class: "w-max min-w-full max-w-none news-markdown-table",
                            SafeMarkdownNodes { nodes: children.clone() }
                        }
                    }
                },
                SafeMarkdownNode::TableHead(children) => rsx! {
                    thead { class: "news-markdown-table-head",
                        tr { class: "news-markdown-table-row",
                            SafeMarkdownNodes { nodes: children.clone() }
                        }
                    }
                },
                SafeMarkdownNode::TableRow(children) => rsx! {
                    tr { class: "news-markdown-table-row",
                        SafeMarkdownNodes { nodes: children.clone() }
                    }
                },
                SafeMarkdownNode::TableCell { alignment, header, children } => {
                    let class = alignment_class(*alignment);
                    if *header {
                        rsx! {
                            th { class: class, SafeMarkdownNodes { nodes: children.clone() } }
                        }
                    } else {
                        rsx! {
                            td { class: class, SafeMarkdownNodes { nodes: children.clone() } }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn NewsNotFound() -> Element {
    rsx! {
        div { class: "news-detail-not-found container page-content flex min-h-[60vh] items-center justify-center",
            section { class: "card card-glass max-w-xl p-8 sm:p-12 text-center", aria_labelledby: "news-not-found-title",
                div { class: "mx-auto mb-4 text-cyan-500", Icon { name: "newspaper".to_string(), size: Some(40) } }
                h1 { id: "news-not-found-title", class: "text-2xl font-bold", "Article not found" }
                p { class: "mt-3 text-sm text-muted-foreground", "This article is not available as published content." }
                a { class: "btn btn-primary mt-6", href: "/news", "Browse all news" }
            }
        }
    }
}

#[component]
fn NewsDetailError(retry_href: String) -> Element {
    rsx! {
        div { class: "news-detail-error container page-content flex min-h-[60vh] items-center justify-center",
            section { class: "card card-glass max-w-xl p-8 sm:p-12 text-center", role: "alert",
                div { class: "mx-auto mb-4 text-cyan-500", Icon { name: "triangle-alert".to_string(), size: Some(40) } }
                h1 { class: "text-2xl font-bold", "Article temporarily unavailable" }
                p { class: "mt-3 text-sm text-muted-foreground", "We could not load this published article. No default article is being shown." }
                div { class: "mt-6 flex flex-wrap justify-center gap-3",
                    a { class: "btn btn-primary", href: retry_href, "Try again" }
                    a { class: "btn btn-outline", href: "/news", "Back to news" }
                }
            }
        }
    }
}

fn alignment_class(alignment: SafeTableAlignment) -> &'static str {
    match alignment {
        SafeTableAlignment::None => "text-left news-markdown-table-cell",
        SafeTableAlignment::Left => "text-left news-markdown-table-cell-left",
        SafeTableAlignment::Center => "text-center news-markdown-table-cell-center",
        SafeTableAlignment::Right => "text-right news-markdown-table-cell-right",
    }
}

fn read_minutes(visible_text: &str) -> usize {
    visible_text.split_whitespace().count().div_ceil(200).max(1)
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS
}

fn parse_safe_markdown(body: &str) -> SafeMarkdownContent {
    if body.len() > MAX_BODY_BYTES {
        return SafeMarkdownContent::Plaintext(bounded_plaintext(body));
    }
    match parse_safe_markdown_tree(body) {
        Ok(nodes) => {
            let visible_text = visible_text_from_nodes(&nodes);
            SafeMarkdownContent::Rich {
                nodes,
                visible_text,
            }
        }
        Err(_) => SafeMarkdownContent::Plaintext(bounded_plaintext(body)),
    }
}

fn parse_safe_markdown_tree(body: &str) -> Result<Vec<SafeMarkdownNode>, MarkdownLimit> {
    let mut stack = vec![MarkdownFrame {
        end: None,
        kind: MarkdownFrameKind::Root,
        children: Vec::new(),
    }];
    let mut event_count = 0_usize;

    for event in Parser::new_ext(body, markdown_options()) {
        event_count += 1;
        if event_count > MAX_MARKDOWN_EVENTS {
            return Err(MarkdownLimit);
        }
        match event {
            Event::Start(tag) => start_markdown_frame(&mut stack, tag)?,
            Event::End(end) => close_markdown_frame(&mut stack, end)?,
            Event::Text(text) => append_markdown_text(&mut stack, text.into_string())?,
            Event::Code(code) => {
                append_markdown_node(&mut stack, SafeMarkdownNode::InlineCode(code.into_string()))?
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                append_markdown_text(&mut stack, html.into_string())?
            }
            Event::FootnoteReference(label) => {
                append_markdown_text(&mut stack, format!("[^{}]", label.into_string()))?
            }
            Event::SoftBreak => append_markdown_node(&mut stack, SafeMarkdownNode::SoftBreak)?,
            Event::HardBreak => append_markdown_node(&mut stack, SafeMarkdownNode::HardBreak)?,
            Event::Rule => append_markdown_node(&mut stack, SafeMarkdownNode::HorizontalRule)?,
            Event::TaskListMarker(checked) => {
                append_markdown_node(&mut stack, SafeMarkdownNode::TaskMarker(checked))?
            }
        }
    }
    if stack.len() != 1 {
        return Err(MarkdownLimit);
    }
    stack.pop().map(|root| root.children).ok_or(MarkdownLimit)
}

fn start_markdown_frame(stack: &mut Vec<MarkdownFrame>, tag: Tag<'_>) -> Result<(), MarkdownLimit> {
    if stack.len() > MAX_MARKDOWN_NESTING {
        return Err(MarkdownLimit);
    }
    let end = tag.to_end();
    let kind = match tag {
        Tag::Paragraph => MarkdownFrameKind::Paragraph,
        Tag::Heading { level, .. } => MarkdownFrameKind::Heading(demoted_heading(level)),
        Tag::BlockQuote => MarkdownFrameKind::BlockQuote,
        Tag::CodeBlock(kind) => MarkdownFrameKind::CodeBlock {
            language: safe_code_language(kind),
            code: String::new(),
        },
        Tag::HtmlBlock | Tag::FootnoteDefinition(_) | Tag::MetadataBlock(_) => {
            MarkdownFrameKind::Transparent
        }
        Tag::List(start) => MarkdownFrameKind::List(start),
        Tag::Item => MarkdownFrameKind::ListItem,
        Tag::Table(alignments) => {
            if alignments.len() > MAX_TABLE_COLUMNS {
                return Err(MarkdownLimit);
            }
            MarkdownFrameKind::Table {
                alignments: alignments
                    .into_iter()
                    .map(SafeTableAlignment::from)
                    .collect(),
                rows: 0,
            }
        }
        Tag::TableHead => {
            increment_table_rows(stack)?;
            MarkdownFrameKind::TableHead { cells: 0 }
        }
        Tag::TableRow => {
            increment_table_rows(stack)?;
            MarkdownFrameKind::TableRow { cells: 0 }
        }
        Tag::TableCell => {
            let (cell_index, header) = match stack.last_mut().map(|frame| &mut frame.kind) {
                Some(MarkdownFrameKind::TableHead { cells }) => {
                    let index = *cells;
                    *cells += 1;
                    (index, true)
                }
                Some(MarkdownFrameKind::TableRow { cells }) => {
                    let index = *cells;
                    *cells += 1;
                    (index, false)
                }
                _ => return Err(MarkdownLimit),
            };
            if cell_index >= MAX_TABLE_COLUMNS {
                return Err(MarkdownLimit);
            }
            let alignment = stack
                .iter()
                .rev()
                .find_map(|frame| match &frame.kind {
                    MarkdownFrameKind::Table { alignments, .. } => {
                        alignments.get(cell_index).copied()
                    }
                    _ => None,
                })
                .unwrap_or(SafeTableAlignment::None);
            MarkdownFrameKind::TableCell { alignment, header }
        }
        Tag::Emphasis => MarkdownFrameKind::Emphasis,
        Tag::Strong => MarkdownFrameKind::Strong,
        Tag::Strikethrough => MarkdownFrameKind::Strikethrough,
        Tag::Link {
            dest_url, title, ..
        } => {
            let title = safe_link_title(title.as_ref())?;
            MarkdownFrameKind::Link {
                target: safe_link_target(dest_url.as_ref()),
                title,
            }
        }
        Tag::Image { title, .. } => {
            safe_link_title(title.as_ref())?;
            MarkdownFrameKind::Image
        }
    };
    stack.push(MarkdownFrame {
        end: Some(end),
        kind,
        children: Vec::new(),
    });
    Ok(())
}

fn close_markdown_frame(stack: &mut Vec<MarkdownFrame>, end: TagEnd) -> Result<(), MarkdownLimit> {
    let frame = stack.pop().ok_or(MarkdownLimit)?;
    if frame.end != Some(end) {
        return Err(MarkdownLimit);
    }
    let nodes = match frame.kind {
        MarkdownFrameKind::Root => return Err(MarkdownLimit),
        MarkdownFrameKind::Paragraph => {
            vec![SafeMarkdownNode::Paragraph(frame.children)]
        }
        MarkdownFrameKind::Heading(level) => vec![SafeMarkdownNode::Heading {
            level,
            children: frame.children,
        }],
        MarkdownFrameKind::BlockQuote => {
            vec![SafeMarkdownNode::BlockQuote(frame.children)]
        }
        MarkdownFrameKind::List(start) => vec![SafeMarkdownNode::List {
            start,
            children: frame.children,
        }],
        MarkdownFrameKind::ListItem => {
            vec![SafeMarkdownNode::ListItem(frame.children)]
        }
        MarkdownFrameKind::Emphasis => {
            vec![SafeMarkdownNode::Emphasis(frame.children)]
        }
        MarkdownFrameKind::Strong => vec![SafeMarkdownNode::Strong(frame.children)],
        MarkdownFrameKind::Strikethrough => {
            vec![SafeMarkdownNode::Strikethrough(frame.children)]
        }
        MarkdownFrameKind::Link { target, title } => match target {
            Some(target) => vec![SafeMarkdownNode::Link {
                target,
                title,
                children: frame.children,
            }],
            None => frame.children,
        },
        MarkdownFrameKind::Image => {
            vec![SafeMarkdownNode::Text(visible_text_from_nodes(
                &frame.children,
            ))]
        }
        MarkdownFrameKind::CodeBlock { language, code } => {
            vec![SafeMarkdownNode::CodeBlock { language, code }]
        }
        MarkdownFrameKind::Table { .. } => {
            vec![SafeMarkdownNode::Table(frame.children)]
        }
        MarkdownFrameKind::TableHead { .. } => {
            vec![SafeMarkdownNode::TableHead(frame.children)]
        }
        MarkdownFrameKind::TableRow { .. } => {
            vec![SafeMarkdownNode::TableRow(frame.children)]
        }
        MarkdownFrameKind::TableCell { alignment, header } => vec![SafeMarkdownNode::TableCell {
            alignment,
            header,
            children: frame.children,
        }],
        MarkdownFrameKind::Transparent => frame.children,
    };
    let parent = stack.last_mut().ok_or(MarkdownLimit)?;
    parent.children.extend(nodes);
    Ok(())
}

fn append_markdown_text(stack: &mut [MarkdownFrame], text: String) -> Result<(), MarkdownLimit> {
    let frame = stack.last_mut().ok_or(MarkdownLimit)?;
    if let MarkdownFrameKind::CodeBlock { code, .. } = &mut frame.kind {
        if code.len().saturating_add(text.len()) > MAX_CODE_BLOCK_BYTES {
            return Err(MarkdownLimit);
        }
        code.push_str(&text);
    } else {
        frame.children.push(SafeMarkdownNode::Text(text));
    }
    Ok(())
}

fn append_markdown_node(
    stack: &mut [MarkdownFrame],
    node: SafeMarkdownNode,
) -> Result<(), MarkdownLimit> {
    let frame = stack.last_mut().ok_or(MarkdownLimit)?;
    if matches!(frame.kind, MarkdownFrameKind::CodeBlock { .. }) {
        return Err(MarkdownLimit);
    }
    frame.children.push(node);
    Ok(())
}

fn increment_table_rows(stack: &mut [MarkdownFrame]) -> Result<(), MarkdownLimit> {
    let table = stack
        .iter_mut()
        .rev()
        .find_map(|frame| match &mut frame.kind {
            MarkdownFrameKind::Table { rows, .. } => Some(rows),
            _ => None,
        })
        .ok_or(MarkdownLimit)?;
    *table += 1;
    if *table > MAX_TABLE_ROWS {
        return Err(MarkdownLimit);
    }
    Ok(())
}

fn demoted_heading(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 2,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn safe_code_language(kind: CodeBlockKind<'_>) -> Option<String> {
    let CodeBlockKind::Fenced(info) = kind else {
        return None;
    };
    let token = info.split_whitespace().next()?;
    (token.len() <= 32
        && !token.is_empty()
        && token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'+' | b'.' | b'-')))
    .then(|| token.to_string())
}

fn safe_link_title(value: &str) -> Result<Option<String>, MarkdownLimit> {
    if value.chars().count() > MAX_LINK_TITLE_CHARS {
        return Err(MarkdownLimit);
    }
    Ok((!value.is_empty() && !value.chars().any(char::is_control)).then(|| value.to_string()))
}

fn safe_link_target(value: &str) -> Option<SafeLinkTarget> {
    if value.is_empty()
        || value.len() > 2_048
        || value.chars().any(char::is_control)
        || value.contains('\\')
    {
        return None;
    }
    if value.starts_with('/') {
        if value.starts_with("//") {
            return None;
        }
        let base = Url::parse("https://epsx.invalid/").ok()?;
        let parsed = base.join(value).ok()?;
        if parsed.scheme() == "https"
            && parsed.host_str() == Some("epsx.invalid")
            && parsed.username().is_empty()
            && parsed.password().is_none()
        {
            return Some(SafeLinkTarget::Internal(value.to_string()));
        }
        return None;
    }
    let authority = canonical_https_authority(value)?;
    if authority.contains('@') {
        return None;
    }
    let url = Url::parse(value).ok()?;
    (url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none())
    .then(|| SafeLinkTarget::External(value.to_string()))
}

fn canonical_https_authority(value: &str) -> Option<&str> {
    let (scheme, rest) = value.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    (!authority.is_empty()).then_some(authority)
}

fn visible_text_from_nodes(nodes: &[SafeMarkdownNode]) -> String {
    let mut visible = String::new();
    append_visible_text(nodes, &mut visible);
    visible.trim().to_string()
}

fn append_visible_text(nodes: &[SafeMarkdownNode], visible: &mut String) {
    for node in nodes {
        match node {
            SafeMarkdownNode::Text(value) | SafeMarkdownNode::InlineCode(value) => {
                visible.push_str(value);
            }
            SafeMarkdownNode::CodeBlock { code, .. } => {
                visible.push_str(code);
                visible.push(' ');
            }
            SafeMarkdownNode::SoftBreak | SafeMarkdownNode::HardBreak => visible.push(' '),
            SafeMarkdownNode::HorizontalRule | SafeMarkdownNode::TaskMarker(_) => {}
            SafeMarkdownNode::Emphasis(children)
            | SafeMarkdownNode::Strong(children)
            | SafeMarkdownNode::Strikethrough(children)
            | SafeMarkdownNode::Link { children, .. }
            | SafeMarkdownNode::TableCell { children, .. } => {
                append_visible_text(children, visible);
            }
            SafeMarkdownNode::Paragraph(children)
            | SafeMarkdownNode::Heading { children, .. }
            | SafeMarkdownNode::BlockQuote(children)
            | SafeMarkdownNode::List { children, .. }
            | SafeMarkdownNode::ListItem(children)
            | SafeMarkdownNode::Table(children)
            | SafeMarkdownNode::TableHead(children)
            | SafeMarkdownNode::TableRow(children) => {
                append_visible_text(children, visible);
                visible.push(' ');
            }
        }
    }
}

fn bounded_plaintext(value: &str) -> String {
    let mut end = value.len().min(MAX_BODY_BYTES);
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

impl From<Alignment> for SafeTableAlignment {
    fn from(value: Alignment) -> Self {
        match value {
            Alignment::None => Self::None,
            Alignment::Left => Self::Left,
            Alignment::Center => Self::Center,
            Alignment::Right => Self::Right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(slug: &str, outcome: serde_json::Value) -> PageContext {
        let mut ctx = PageContext {
            path: format!("/news/{slug}"),
            ..Default::default()
        };
        ctx.params.insert("slug".to_string(), slug.to_string());
        ctx.params
            .insert("data_news_post".to_string(), outcome.to_string());
        ctx
    }

    fn article(title: &str, body: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "article-1",
            "slug": "live-article",
            "title": title,
            "summary": "Live article summary",
            "body": body,
            "cover_image_url": null,
            "author": null,
            "published_at": "July 22, 2026",
            "tags": ["engineering"]
        })
    }

    fn render_markdown(body: &str) -> String {
        dioxus_ssr::render_element(rsx! {
            SafeMarkdownView { content: parse_safe_markdown(body) }
        })
    }

    #[test]
    fn ready_article_uses_live_title_body_metadata_and_escaped_html() {
        let ctx = context(
            "live-article",
            serde_json::json!({
                "state": "ready",
                "article": article(
                    "Live <script>alert(1)</script>",
                    "## Update\n\n<p>Safe <img src=x onerror=alert(1)> body</p>"
                )
            }),
        );
        let (meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        assert_eq!(meta.title, "Live <script>alert(1)</script> — EPSX News");
        assert_eq!(meta.description, "Live article summary");
        assert!(html.contains("Live "));
        assert!(html.contains("alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&#60;p&#62;Safe"), "{html}");
        assert!(html.contains("onerror=alert(1)"));
        assert!(!html.contains("<img src=x"));
        assert!(html.contains("July 22, 2026"));
        assert!(!html.contains("Related articles"));
        assert!(!html.contains("Welcome to EPSX"));
    }

    #[test]
    fn supported_markdown_renders_explicit_blocks_and_inlines() {
        let html = render_markdown(
            "# H1\n\n### H3\n\nParagraph with *em*, **strong**, ~~gone~~, `inline`, and  \\\na break.\n\n> quote\n\n---\n\n1. first\n   - nested\n   - [x] done\n\n[reference][safe-ref]\n\n[safe-ref]: https://reference.example/item\n\n```rust extra\nfn main() {}\n```\n\n```bad/lang\nunsafe_language()\n```\n\n| plain | left | center | right |\n| --- | :--- | :---: | ---: |\n| zero | one | middle | two |\n",
        );
        assert!(html.contains("<h2 class=\"news-markdown-heading-2\">H1</h2>"));
        assert!(html.contains("<h3 class=\"news-markdown-heading-3\">H3</h3>"));
        assert!(html.contains("<em class=\"news-markdown-emphasis\">em</em>"));
        assert!(html.contains("<strong class=\"news-markdown-strong\">strong</strong>"));
        assert!(html.contains("<del class=\"news-markdown-strikethrough\">gone</del>"));
        assert!(html.contains("<code class=\"news-markdown-inline-code\">inline</code>"));
        assert!(html.contains("<br"));
        assert!(html.contains("<blockquote"));
        assert!(html.contains("<hr"));
        assert!(html.contains("<ol"));
        assert!(html.contains("<ul"));
        assert!(html.contains("disabled"));
        assert!(html.contains("checked"));
        assert!(html.contains("href=\"https://reference.example/item\""));
        assert!(html.contains("news-markdown-code-language\">rust</span>"));
        assert!(!html.contains("news-markdown-code-language\">bad/lang</span>"));
        assert!(html.contains("unsafe_language()"));
        assert!(html.contains("<table"));
        assert!(html.contains("dark:prose-invert"));
        assert!(html.contains("max-w-full overflow-x-auto news-markdown-table-scroll"));
        assert!(html.contains("w-max min-w-full max-w-none news-markdown-table"));
        assert!(html.contains("<th class=\"text-left news-markdown-table-cell\">plain</th>"));
        assert!(html.contains("<th class=\"text-left news-markdown-table-cell-left\">"));
        assert!(html.contains("<th class=\"text-center news-markdown-table-cell-center\">"));
        assert!(html.contains("<td class=\"text-right news-markdown-table-cell-right\">"));
    }

    #[test]
    fn links_html_images_escapes_and_entities_are_inert() {
        let html = render_markdown(
            "[internal](/news/item \"Local\") [external](https://example.com/a) \
             [credential](https://user@credential.example/x) \
             [empty-credential](https://@empty-credential.example/x) \
             [mixed-credential](HtTpS://user@mixed-credential.example/x) \
             [mixed-empty-credential](HTTPS://@mixed-empty.example/x) \
             [mixed-valid](HTTPS://mixed-valid.example/x) \
             [extra-slashes](https:////@extra-slashes.example/x) \
             [one-slash](https:/@one-slash.example/x) \
             [no-slashes](https:@no-slashes.example/x) \
             [protocol](//protocol.example/x) [http](http://http.example/x) \
             [data](data:text/html,boom) [mailto](mailto:person@example.com) \
             [bad](javascript:alert(1)) <https://example.org/x> \
             <user@example.com>\n\n![Alt **only**](https://evil.example/image.png) \
             <script>alert('x')</script><iframe src=\"https://evil.example\"></iframe> \
             <svg onload=\"alert(1)\"></svg><img src=x onerror=alert(1)> \
             Fish &amp; Chips and \\*literal\\*",
        );
        assert!(html.contains("href=\"/news/item\""));
        assert!(html.contains("title=\"Local\""));
        assert!(html.contains("href=\"https://example.com/a\""));
        assert!(html.contains("href=\"https://example.org/x\""));
        assert!(html.contains("href=\"HTTPS://mixed-valid.example/x\""));
        assert!(html.contains("rel=\"nofollow noopener noreferrer\""));
        assert!(!html.contains("href=\"javascript:"));
        assert!(!html.contains("href=\"https://user@"));
        assert!(!html.contains("href=\"https://@"));
        assert!(!html.contains("href=\"HtTpS://user@"));
        assert!(!html.contains("href=\"HTTPS://@"));
        assert!(!html.contains("href=\"https:////@"));
        assert!(!html.contains("href=\"https:/@"));
        assert!(!html.contains("href=\"https:@"));
        assert!(!html.contains("href=\"//protocol.example"));
        assert!(!html.contains("href=\"http://http.example"));
        assert!(!html.contains("href=\"data:"));
        assert!(!html.contains("href=\"mailto:"));
        for label in [
            "credential",
            "empty-credential",
            "mixed-credential",
            "mixed-empty-credential",
            "extra-slashes",
            "one-slash",
            "no-slashes",
            "protocol",
            "http",
            "data",
            "mailto",
            "bad",
        ] {
            assert!(html.contains(label), "unsafe link label was lost: {label}");
        }
        assert!(html.contains("bad"));
        assert!(html.contains("user@example.com"));
        assert!(html.contains("Alt only"), "{html}");
        assert!(!html.contains("<img"));
        assert!(html.contains("&#60;script&#62;"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<iframe"));
        assert!(!html.contains("<svg"));
        assert!(html.contains("Fish &#38; Chips"));
        assert!(html.contains("*literal*"));
    }

    #[test]
    fn structural_limits_discard_rich_nodes_for_bounded_plaintext() {
        let nested = format!("{}x", "> ".repeat(MAX_MARKDOWN_NESTING + 1));
        let html = render_markdown(&nested);
        assert!(
            html.contains("data-news-markdown-fallback=\"true\""),
            "{html}"
        );
        assert!(html.contains("dark:prose-invert"));
        assert!(!html.contains("<em"));

        let oversized_code = format!("```\n{}\n```", "word ".repeat(40_000));
        let content = parse_safe_markdown(&oversized_code);
        let SafeMarkdownContent::Plaintext(plaintext) = &content else {
            panic!("oversized code must discard the rich tree");
        };
        assert_eq!(plaintext.len(), oversized_code.len());
        assert!(plaintext.ends_with("\n```"));
        assert_eq!(
            read_minutes(content.visible_text()),
            read_minutes(&oversized_code)
        );
        assert!(read_minutes(content.visible_text()) > 100);
        let html = render_markdown(&oversized_code);
        assert!(html.contains("data-news-markdown-fallback=\"true\""));
        assert!(!html.contains("news-markdown-code-block"));

        let too_many_events = "word\n\n".repeat(MAX_MARKDOWN_EVENTS / 3 + 1);
        let content = parse_safe_markdown(&too_many_events);
        if let SafeMarkdownContent::Plaintext(text) = content {
            assert_eq!(text.len(), too_many_events.len());
        } else {
            panic!("event-limited content must fall back to plaintext");
        }

        let columns = (0..=MAX_TABLE_COLUMNS)
            .map(|column| format!("column-{column}"))
            .collect::<Vec<_>>()
            .join(" | ");
        let separators = (0..=MAX_TABLE_COLUMNS)
            .map(|_| "---")
            .collect::<Vec<_>>()
            .join(" | ");
        let oversized_table = format!("| {columns} |\n| {separators} |\n");
        assert!(matches!(
            parse_safe_markdown(&oversized_table),
            SafeMarkdownContent::Plaintext(_)
        ));

        let oversized_rows = format!(
            "| column |\n| --- |\n{}",
            "| value |\n".repeat(MAX_TABLE_ROWS + 1)
        );
        assert!(matches!(
            parse_safe_markdown(&oversized_rows),
            SafeMarkdownContent::Plaintext(_)
        ));

        let oversized_title = format!(
            "[label](/news/item \"{}\")",
            "x".repeat(MAX_LINK_TITLE_CHARS + 1)
        );
        assert!(matches!(
            parse_safe_markdown(&oversized_title),
            SafeMarkdownContent::Plaintext(_)
        ));
    }

    #[test]
    fn unsafe_metadata_and_field_bounds_render_retryable_error() {
        let mut invalid_cases = Vec::new();

        let mut id = article("Valid", "Body");
        id["id"] = serde_json::json!("x".repeat(129));
        invalid_cases.push(id);

        let mut title = article("Valid", "Body");
        title["title"] = serde_json::json!("x".repeat(201));
        invalid_cases.push(title);

        let mut summary = article("Valid", "Body");
        summary["summary"] = serde_json::json!("x".repeat(501));
        invalid_cases.push(summary);

        let mut author = article("Valid", "Body");
        author["author"] = serde_json::json!(" ");
        invalid_cases.push(author);

        let mut cover = article("Valid", "Body");
        cover["cover_image_url"] = serde_json::json!("https://user@example.com/image.png");
        invalid_cases.push(cover);

        let mut fragmented_cover = article("Valid", "Body");
        fragmented_cover["cover_image_url"] =
            serde_json::json!("https://example.com/image.png#fragment");
        invalid_cases.push(fragmented_cover);

        let mut empty_credential_cover = article("Valid", "Body");
        empty_credential_cover["cover_image_url"] =
            serde_json::json!("https://@example.com/image.png");
        invalid_cases.push(empty_credential_cover);

        let mut mixed_credential_cover = article("Valid", "Body");
        mixed_credential_cover["cover_image_url"] =
            serde_json::json!("HtTpS://user@example.com/image.png");
        invalid_cases.push(mixed_credential_cover);

        let mut mixed_empty_credential_cover = article("Valid", "Body");
        mixed_empty_credential_cover["cover_image_url"] =
            serde_json::json!("HTTPS://@example.com/image.png");
        invalid_cases.push(mixed_empty_credential_cover);

        for malformed_cover in [
            "https:////@example.com/image.png",
            "https:/@example.com/image.png",
            "https:@example.com/image.png",
        ] {
            let mut malformed = article("Valid", "Body");
            malformed["cover_image_url"] = serde_json::json!(malformed_cover);
            invalid_cases.push(malformed);
        }

        let mut protocol_relative_cover = article("Valid", "Body");
        protocol_relative_cover["cover_image_url"] = serde_json::json!("//example.com/image.png");
        invalid_cases.push(protocol_relative_cover);

        let mut insecure_cover = article("Valid", "Body");
        insecure_cover["cover_image_url"] = serde_json::json!("http://example.com/image.png");
        invalid_cases.push(insecure_cover);

        let mut date = article("Valid", "Body");
        date["published_at"] = serde_json::json!("February 30, 2026");
        invalid_cases.push(date);

        let mut year_zero = article("Valid", "Body");
        year_zero["published_at"] = serde_json::json!("January 1, 0000");
        invalid_cases.push(year_zero);

        let mut extended_year = article("Valid", "Body");
        extended_year["published_at"] = serde_json::json!("January 1, +10000");
        invalid_cases.push(extended_year);

        let mut oversized_body = article("Valid", "Body");
        oversized_body["body"] = serde_json::json!("x".repeat(MAX_BODY_BYTES + 1));
        invalid_cases.push(oversized_body);

        for article in invalid_cases {
            let (_, element) = render(&context(
                "live-article",
                serde_json::json!({"state": "ready", "article": article}),
            ));
            assert!(dioxus_ssr::render_element(element).contains("temporarily unavailable"));
        }
    }

    #[test]
    fn mixed_case_canonical_https_cover_is_accepted_verbatim() {
        let mut valid = article("Valid", "Body");
        valid["cover_image_url"] = serde_json::json!("HTTPS://example.com/images/news.png");
        let (_, element) = render(&context(
            "live-article",
            serde_json::json!({"state": "ready", "article": valid}),
        ));
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("src=\"HTTPS://example.com/images/news.png\""));
        assert!(!html.contains("temporarily unavailable"));
    }

    #[test]
    fn display_dates_accept_canonical_year_boundaries() {
        assert!(valid_display_date("January 1, 0001"));
        assert!(valid_display_date("December 31, 9999"));
        assert!(!valid_display_date("January 1, 0000"));
        assert!(!valid_display_date("January 1, +10000"));
    }

    #[test]
    fn read_time_uses_visible_owned_markdown_text() {
        let body = format!(
            "# Heading\n\n{} ![visible alt](https://example.com/image.png)",
            "word ".repeat(399)
        );
        let (_, element) = render(&context(
            "live-article",
            serde_json::json!({
                "state": "ready",
                "article": article("Visible read time", &body)
            }),
        ));
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("3 min read"));
    }

    #[test]
    fn not_found_is_explicit_and_never_synthesizes_an_article() {
        let (meta, element) = render(&context(
            "missing-article",
            serde_json::json!({"state": "not_found"}),
        ));
        let html = dioxus_ssr::render_element(element);
        assert_eq!(meta.status, PageStatus::NotFound);
        assert!(html.contains("Article not found"));
        assert!(!html.contains("coming soon"));
        assert!(!html.contains("Welcome to EPSX"));
    }

    #[test]
    fn every_detail_state_defers_the_main_landmark_to_the_shared_shell() {
        let contexts = [
            context(
                "live-article",
                serde_json::json!({
                    "state": "ready",
                    "article": article("Live article", "Published body")
                }),
            ),
            context("missing-article", serde_json::json!({"state": "not_found"})),
            context(
                "failed-article",
                serde_json::json!({"state": "error", "code": "dependency_unavailable"}),
            ),
        ];

        for ctx in contexts {
            let (_, element) = render(&ctx);
            let html = dioxus_ssr::render_element(element);
            assert!(
                !html.contains("<main"),
                "news detail page fragment must not nest a main landmark for {}: {html}",
                ctx.path
            );
        }
    }

    #[test]
    fn missing_malformed_or_slug_mismatched_outcome_renders_retryable_error() {
        let mut missing = PageContext {
            path: "/news/live-article".to_string(),
            ..Default::default()
        };
        missing
            .params
            .insert("slug".to_string(), "live-article".to_string());
        let (_, element) = render(&missing);
        assert!(dioxus_ssr::render_element(element).contains("temporarily unavailable"));

        let (_, mismatched) = render(&context(
            "live-article",
            serde_json::json!({
                "state": "ready",
                "article": {
                    "id": null,
                    "slug": "another-article",
                    "title": "Wrong owner",
                    "summary": null,
                    "body": "Body",
                    "cover_image_url": null,
                    "author": null,
                    "published_at": null,
                    "tags": []
                }
            }),
        ));
        let html = dioxus_ssr::render_element(mismatched);
        assert!(html.contains("temporarily unavailable"));
        assert!(!html.contains("Wrong owner"));

        let mut malformed_date = article("Malformed date", "Body");
        malformed_date["published_at"] = serde_json::json!("not-a-date");
        let (_, malformed_date) = render(&context(
            "live-article",
            serde_json::json!({"state": "ready", "article": malformed_date}),
        ));
        assert!(dioxus_ssr::render_element(malformed_date).contains("temporarily unavailable"));
    }
}
