use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Missing prop: {0}")]
    MissingProp(String),
}

pub type Result<T> = std::result::Result<T, RenderError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub r#type: String,
    pub props: serde_json::Value,
    #[serde(default)]
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub seo: serde_json::Value,
    #[serde(default)]
    pub theme: Option<String>,
}

pub fn render_markdown(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    let parser = Parser::new_ext(md, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn get_string(props: &serde_json::Value, key: &str, default: &str) -> String {
    props.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

pub fn get_u32(props: &serde_json::Value, key: &str, default: u32) -> u32 {
    props.get(key).and_then(|v| v.as_u64()).map(|v| v as u32).unwrap_or(default)
}

pub fn get_array(props: &serde_json::Value, key: &str) -> Vec<serde_json::Value> {
    props.get(key).and_then(|v| v.as_array()).cloned().unwrap_or_default()
}

pub fn get_bool(props: &serde_json::Value, key: &str, default: bool) -> bool {
    props.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

pub fn render_block(block: &Block) -> String {
    match block.r#type.as_str() {
        "hero" => render_hero(&block.props),
        "features" => render_features(&block.props),
        "pricing" => render_pricing(&block.props),
        "testimonial" => render_testimonial(&block.props),
        "cta-banner" => render_cta(&block.props),
        "blog-list" => render_blog_list(&block.props),
        "rich-text" => render_rich_text(&block.props),
        "custom-html" => render_custom_html(&block.props),
        other => format!("<!-- unknown block: {} -->", other),
    }
}

pub fn render_page(page: &Page) -> String {
    let blocks_html: String = page.blocks.iter().map(render_block).collect();
    format!(
        r#"<main data-page-slug="{}" data-page-title="{}">{}</main>"#,
        page.slug, page.title, blocks_html
    )
}

fn render_hero(p: &serde_json::Value) -> String {
    let title = get_string(p, "title", "");
    let subtitle = get_string(p, "subtitle", "");
    let cta_text = get_string(p, "ctaText", "Get Started");
    let cta_link = get_string(p, "ctaLink", "/");
    let bg = get_string(p, "background", "gradient");
    let align = get_string(p, "alignment", "center");
    format!(
        r#"<section class="hero hero-bg-{bg} align-{align}" style="padding:96px 24px;text-align:{align};">
<h1 style="font-size:48px;font-weight:bold;margin:0 0 16px;">{title}</h1>
<p style="font-size:20px;opacity:0.7;margin:0 0 32px;">{subtitle}</p>
<a href="{cta_link}" style="display:inline-block;background:#3b82f6;color:#fff;padding:12px 32px;border-radius:8px;text-decoration:none;font-weight:600;">{cta_text}</a>
</section>"#,
        title = title,
        subtitle = subtitle,
        cta_link = cta_link,
        cta_text = cta_text,
        bg = bg,
        align = align
    )
}

fn render_features(p: &serde_json::Value) -> String {
    let title = get_string(p, "title", "");
    let items = get_array(p, "items");
    let columns = get_u32(p, "columns", 3);
    let cards: String = items.iter().map(|it| {
        let t = it.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let d = it.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let icon = it.get("icon").and_then(|v| v.as_str()).unwrap_or("");
        format!(
            r#"<div class="feature-card" style="background:rgba(17,24,39,0.6);padding:24px;border-radius:12px;border:1px solid #1f2937;">
<h3 style="font-size:18px;font-weight:600;margin:0 0 8px;">{icon} {t}</h3>
<p style="opacity:0.7;margin:0;">{d}</p>
</div>"#,
            icon = icon,
            t = t,
            d = d
        )
    }).collect();
    format!(
        r#"<section class="features" style="padding:64px 24px;">
<div style="max-width:1200px;margin:0 auto;">
<h2 style="font-size:32px;font-weight:bold;text-align:center;margin:0 0 48px;">{title}</h2>
<div class="features-grid" style="display:grid;grid-template-columns:repeat({cols},minmax(0,1fr));gap:24px;">
{cards}
</div>
</div>
</section>"#,
        title = title,
        cards = cards,
        cols = columns
    )
}

fn render_pricing(p: &serde_json::Value) -> String {
    let plans = get_array(p, "plans");
    let cards: String = plans.iter().map(|plan| {
        let name = plan.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let price = plan.get("price").and_then(|v| v.as_str()).unwrap_or("");
        let cta = plan.get("ctaText").and_then(|v| v.as_str()).unwrap_or("Subscribe");
        let highlighted = plan.get("highlighted").and_then(|v| v.as_bool()).unwrap_or(false);
        let features_arr = plan.get("features").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let features: String = features_arr.iter().map(|f| {
            let t = f.as_str().unwrap_or("");
            format!("<li style=\"padding:8px 0;opacity:0.7;\">{t}</li>")
        }).collect();
        let border = if highlighted { "2px solid #3b82f6" } else { "1px solid #1f2937" };
        format!(
            r#"<div class="pricing-card" style="background:#111827;padding:24px;border-radius:12px;border:{border};">
<h3 style="font-size:20px;font-weight:600;margin:0 0 8px;">{name}</h3>
<p style="font-size:36px;font-weight:bold;margin:0 0 24px;">{price}</p>
<ul style="list-style:none;padding:0;margin:0 0 24px;">{features}</ul>
<button style="width:100%;background:#3b82f6;color:#fff;padding:12px;border:none;border-radius:8px;font-weight:600;cursor:pointer;">{cta}</button>
</div>"#,
            name = name,
            price = price,
            features = features,
            cta = cta,
            border = border
        )
    }).collect();
    format!(
        r#"<section class="pricing" style="padding:64px 24px;">
<div style="max-width:1200px;margin:0 auto;">
<h2 style="font-size:32px;font-weight:bold;text-align:center;margin:0 0 48px;">Pricing</h2>
<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:24px;max-width:1024px;margin:0 auto;">{cards}</div>
</div>
</section>"#
    )
}

fn render_testimonial(p: &serde_json::Value) -> String {
    let quote = get_string(p, "quote", "");
    let author = get_string(p, "author", "");
    let role = get_string(p, "role", "");
    format!(
        r#"<section class="testimonial" style="padding:64px 24px;text-align:center;">
<blockquote style="font-size:24px;font-style:italic;max-width:800px;margin:0 auto;">&ldquo;{quote}&rdquo;</blockquote>
<p style="margin-top:16px;opacity:0.7;">&mdash; {author}{role_suffix}</p>
</section>"#,
        quote = quote,
        author = author,
        role_suffix = if role.is_empty() { String::new() } else { format!(", {}", role) }
    )
}

fn render_cta(p: &serde_json::Value) -> String {
    let title = get_string(p, "title", "");
    let description = get_string(p, "description", "");
    let cta_text = get_string(p, "ctaText", "Get Started");
    let cta_link = get_string(p, "ctaLink", "/");
    let bg = get_string(p, "background", "gradient");
    let bg_style = match bg.as_str() {
        "primary" => "background:#3b82f6;",
        "dark" => "background:#111827;",
        _ => "background:linear-gradient(135deg,#3b82f6 0%,#8b5cf6 100%);",
    };
    format!(
        r#"<section class="cta" style="padding:64px 24px;text-align:center;{bg}">
<h2 style="font-size:32px;font-weight:bold;margin:0 0 16px;">{title}</h2>
<p style="opacity:0.7;margin:0 0 32px;max-width:600px;margin-left:auto;margin-right:auto;">{description}</p>
<a href="{cta_link}" style="display:inline-block;background:#fff;color:#000;padding:12px 32px;border-radius:8px;text-decoration:none;font-weight:600;">{cta_text}</a>
</section>"#,
        title = title,
        description = description,
        cta_link = cta_link,
        cta_text = cta_text,
        bg = bg_style
    )
}

fn render_blog_list(p: &serde_json::Value) -> String {
    let title = get_string(p, "title", "Latest Posts");
    let posts = get_array(p, "posts");
    let columns = get_u32(p, "columns", 3);
    let show_excerpt = get_bool(p, "showExcerpt", true);
    let cards: String = posts.iter().map(|post| {
        let slug = post.get("slug").and_then(|v| v.as_str()).unwrap_or("");
        let pt = post.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let excerpt = post.get("excerpt").and_then(|v| v.as_str()).unwrap_or("");
        let excerpt_html = if show_excerpt && !excerpt.is_empty() {
            format!(r#"<p style="opacity:0.7;margin:8px 0 0;">{}</p>"#, excerpt)
        } else {
            String::new()
        };
        format!(
            r#"<a href="/blog/{slug}" class="blog-card" style="display:block;background:#111827;padding:24px;border-radius:12px;border:1px solid #1f2937;text-decoration:none;color:inherit;transition:border-color 0.2s;">
<h3 style="font-size:18px;font-weight:600;margin:0;">{pt}</h3>{excerpt_html}
</a>"#,
            slug = slug,
            pt = pt,
            excerpt_html = excerpt_html
        )
    }).collect();
    format!(
        r#"<section class="blog-list" style="padding:64px 24px;">
<div style="max-width:1200px;margin:0 auto;">
<h2 style="font-size:32px;font-weight:bold;margin:0 0 48px;">{title}</h2>
<div style="display:grid;grid-template-columns:repeat({cols},minmax(0,1fr));gap:24px;">{cards}</div>
</div>
</section>"#,
        title = title,
        cards = cards,
        cols = columns
    )
}

fn render_rich_text(p: &serde_json::Value) -> String {
    let md = get_string(p, "markdown", "");
    let html = render_markdown(&md);
    let max_width = get_string(p, "maxWidth", "md");
    let align = get_string(p, "alignment", "left");
    let width = match max_width.as_str() {
        "sm" => "640px",
        "md" => "768px",
        "lg" => "1024px",
        _ => "100%",
    };
    format!(
        r#"<section class="rich-text" style="padding:32px 24px;text-align:{align};">
<div class="prose" style="max-width:{width};margin:0 auto;text-align:left;">{html}</div>
</section>"#,
        html = html,
        align = align,
        width = width
    )
}

fn render_custom_html(p: &serde_json::Value) -> String {
    let html = get_string(p, "html", "");
    let css = get_string(p, "css", "");
    let scripts = get_string(p, "scripts", "");
    let style_tag = if css.is_empty() { String::new() } else { format!(r#"<style>{}</style>"#, css) };
    let script_tag = if scripts.is_empty() { String::new() } else { format!(r#"<script>{}</script>"#, scripts) };
    format!("<div class=\"custom-html\">{style}{html}{script}</div>",
        style = style_tag,
        html = html,
        script = script_tag
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_hero() {
        let block = Block {
            id: "1".to_string(),
            r#type: "hero".to_string(),
            props: serde_json::json!({
                "title": "Hello",
                "subtitle": "World",
                "ctaText": "Click me",
                "ctaLink": "/start"
            }),
            order: 0,
        };
        let html = render_block(&block);
        assert!(html.contains("Hello"));
        assert!(html.contains("Click me"));
    }

    #[test]
    fn test_render_features() {
        let block = Block {
            id: "1".to_string(),
            r#type: "features".to_string(),
            props: serde_json::json!({
                "title": "Why us",
                "items": [
                    {"title": "Fast", "description": "Quick"},
                    {"title": "Cheap", "description": "Free"}
                ],
                "columns": 2
            }),
            order: 0,
        };
        let html = render_block(&block);
        assert!(html.contains("Why us"));
        assert!(html.contains("Fast"));
        assert!(html.contains("Cheap"));
    }

    #[test]
    fn test_render_markdown() {
        let html = render_markdown("# Hello\n\nThis is **bold**.");
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
    }
}
