//! Component library — builder-pattern Rust types for emitting consistent
//! HTML using the design system classes defined in `lib.rs`.
//!
//! Each builder is constructed with `::new(...)`, then chained with variant
//! methods (`.primary()`, `.lg()`, `.glass()`, ...) and finalized with
//! `.render()`. The output is a `String` ready to drop into a page body.
//!
//! Example:
//! ```ignore
//! use epsx_templates::components::{Btn, Card, Badge};
//!
//! let html = format!(
//!   r#"<div class="grid gap-4">
//!     {card}
//!     {btn}
//!   </div>"#,
//!   card = Card::new("Hello", "World").glass().hover().render(),
//!   btn  = Btn::new("Get started").primary().lg().block().render(),
//! );
//! ```

// =====================================================================
// Btn — primary call to action. 9 styles × 4 sizes × 4 shapes.
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtnKind {
    Primary,
    Gradient,
    Brand,
    Cool,
    Outline,
    Ghost,
    Glass,
    Danger,
    Link,
}

impl BtnKind {
    fn cls(self) -> &'static str {
        match self {
            BtnKind::Primary => "btn btn-primary",
            BtnKind::Gradient => "btn btn-gradient",
            BtnKind::Brand => "btn btn-brand",
            BtnKind::Cool => "btn btn-cool",
            BtnKind::Outline => "btn btn-outline",
            BtnKind::Ghost => "btn btn-ghost",
            BtnKind::Glass => "btn btn-glass",
            BtnKind::Danger => "btn btn-danger",
            BtnKind::Link => "btn btn-link",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtnSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl BtnSize {
    fn cls(self) -> &'static str {
        match self {
            BtnSize::Sm => "btn-sm",
            BtnSize::Md => "",
            BtnSize::Lg => "btn-lg",
            BtnSize::Xl => "btn-xl",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Btn {
    label: String,
    kind: BtnKind,
    size: BtnSize,
    href: Option<String>,
    onclick: Option<String>,
    block: bool,
    icon_left: Option<String>,
    icon_right: Option<String>,
    extra_cls: String,
    extra_attrs: String,
}

impl Btn {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: BtnKind::Primary,
            size: BtnSize::Md,
            href: None,
            onclick: None,
            block: false,
            icon_left: None,
            icon_right: None,
            extra_cls: String::new(),
            extra_attrs: String::new(),
        }
    }

    pub fn primary(mut self) -> Self { self.kind = BtnKind::Primary; self }
    pub fn gradient(mut self) -> Self { self.kind = BtnKind::Gradient; self }
    pub fn brand(mut self) -> Self { self.kind = BtnKind::Brand; self }
    pub fn cool(mut self) -> Self { self.kind = BtnKind::Cool; self }
    pub fn outline(mut self) -> Self { self.kind = BtnKind::Outline; self }
    pub fn ghost(mut self) -> Self { self.kind = BtnKind::Ghost; self }
    pub fn glass(mut self) -> Self { self.kind = BtnKind::Glass; self }
    pub fn danger(mut self) -> Self { self.kind = BtnKind::Danger; self }
    pub fn link_style(mut self) -> Self { self.kind = BtnKind::Link; self }

    pub fn sm(mut self) -> Self { self.size = BtnSize::Sm; self }
    pub fn md(mut self) -> Self { self.size = BtnSize::Md; self }
    pub fn lg(mut self) -> Self { self.size = BtnSize::Lg; self }
    pub fn xl(mut self) -> Self { self.size = BtnSize::Xl; self }

    pub fn href(mut self, href: impl Into<String>) -> Self { self.href = Some(href.into()); self }
    pub fn onclick(mut self, js: impl Into<String>) -> Self { self.onclick = Some(js.into()); self }
    pub fn block(mut self) -> Self { self.block = true; self }
    pub fn icon_left(mut self, fa_name: impl Into<String>) -> Self { self.icon_left = Some(fa_name.into()); self }
    pub fn icon_right(mut self, fa_name: impl Into<String>) -> Self { self.icon_right = Some(fa_name.into()); self }
    pub fn cls(mut self, extra: impl Into<String>) -> Self { self.extra_cls = extra.into(); self }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        if !self.extra_attrs.is_empty() { self.extra_attrs.push(' '); }
        self.extra_attrs.push_str(&format!(r#"{}="{}""#, k, html_escape_attr(v)));
        self
    }

    pub fn render(self) -> String {
        let size_cls = self.size.cls();
        let block_cls = if self.block { " w-full justify-center" } else { "" };
        let kind_cls = self.kind.cls();
        let full_cls = format!("{kind_cls} {size_cls}{block_cls} {}", self.extra_cls).trim().to_string();

        let left = self.icon_left.as_deref()
            .map(|i| format!(r#"<i data-lucide="{i}"></i>"#))
            .unwrap_or_default();
        let right = self.icon_right.as_deref()
            .map(|i| format!(r#"<i data-lucide="{i}"></i>"#))
            .unwrap_or_default();

        let attrs = self.extra_attrs;
        let onclick_attr = self.onclick.as_deref()
            .map(|o| format!(r#" onclick="{o}""#))
            .unwrap_or_default();

        if let Some(href) = self.href {
            format!(
                r##"<a href="{href}" class="{cls}"{onclick}{attrs}>{left}<span>{label}</span>{right}</a>"##,
                href = html_escape_attr(&href),
                cls = full_cls,
                onclick = onclick_attr,
                attrs = if attrs.is_empty() { String::new() } else { format!(" {attrs}") },
                left = left,
                label = html_escape_text(&self.label),
                right = right,
            )
        } else {
            format!(
                r##"<button type="button" class="{cls}"{onclick}{attrs}>{left}<span>{label}</span>{right}</button>"##,
                cls = full_cls,
                onclick = onclick_attr,
                attrs = if attrs.is_empty() { String::new() } else { format!(" {attrs}") },
                left = left,
                label = html_escape_text(&self.label),
                right = right,
            )
        }
    }
}

// =====================================================================
// Card — content container. 3 variants: default, glass, insight.
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKind {
    Default,
    Glass,
    Insight,
    Flat,
}

impl CardKind {
    fn cls(self) -> &'static str {
        match self {
            CardKind::Default => "card-glass",
            CardKind::Glass => "card-glass",
            CardKind::Insight => "card-insight",
            CardKind::Flat => "",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    title: Option<String>,
    body: String,
    footer: Option<String>,
    kind: CardKind,
    hover: bool,
    icon: Option<String>,
    icon_color: Option<String>,
    badge: Option<String>,
    extra_cls: String,
}

impl Card {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            body: body.into(),
            footer: None,
            kind: CardKind::Glass,
            hover: false,
            icon: None,
            icon_color: None,
            badge: None,
            extra_cls: String::new(),
        }
    }

    pub fn body_only(body: impl Into<String>) -> Self {
        Self {
            title: None,
            body: body.into(),
            footer: None,
            kind: CardKind::Glass,
            hover: false,
            icon: None,
            icon_color: None,
            badge: None,
            extra_cls: String::new(),
        }
    }

    pub fn default(mut self) -> Self { self.kind = CardKind::Default; self }
    pub fn glass(mut self) -> Self { self.kind = CardKind::Glass; self }
    pub fn insight(mut self) -> Self { self.kind = CardKind::Insight; self }
    pub fn flat(mut self) -> Self { self.kind = CardKind::Flat; self }
    pub fn hover(mut self) -> Self { self.hover = true; self }
    pub fn footer(mut self, html: impl Into<String>) -> Self { self.footer = Some(html.into()); self }
    pub fn icon(mut self, fa_name: impl Into<String>) -> Self { self.icon = Some(fa_name.into()); self }
    pub fn icon_color(mut self, color: impl Into<String>) -> Self { self.icon_color = Some(color.into()); self }
    pub fn badge(mut self, html: impl Into<String>) -> Self { self.badge = Some(html.into()); self }
    pub fn cls(mut self, extra: impl Into<String>) -> Self { self.extra_cls = extra.into(); self }

    pub fn render(self) -> String {
        let kind_cls = self.kind.cls();
        let hover_cls = if self.hover { " hover-scale" } else { "" };
        let base_style = if matches!(self.kind, CardKind::Flat) {
            String::from("padding:1.25rem;")
        } else {
            String::from("padding:1.5rem;")
        };
        let cls = format!("{kind_cls}{hover_cls} {}", self.extra_cls).trim().to_string();

        let title_html = self.title.as_deref().map(|t| {
            let icon_html = self.icon.as_deref().map(|i| {
                let color = self.icon_color.as_deref().unwrap_or("var(--epsx-orange)");
                format!(
                    r#"<div style="width:2.5rem;height:2.5rem;border-radius:0.75rem;display:flex;align-items:center;justify-content:center;background:rgba(249,115,22,0.1);color:{color};font-size:1.125rem;flex-shrink:0;"><i data-lucide="{i}"></i></div>"#,
                    color = color, i = i
                )
            }).unwrap_or_default();
            let badge_html = self.badge.as_deref().unwrap_or("");
            format!(
                r#"<div style="display:flex;align-items:flex-start;justify-content:space-between;gap:1rem;margin-bottom:0.75rem;">
        {icon}
        <div style="flex:1;min-width:0;">
          <h3 style="font-size:1.125rem;font-weight:600;color:var(--text);margin:0;">{t}</h3>
        </div>
        {badge}
      </div>"#,
                icon = icon_html,
                t = html_escape_text(t),
                badge = badge_html
            )
        }).unwrap_or_default();

        let footer_html = self.footer.as_deref()
            .map(|f| format!(r#"<div style="margin-top:1rem;padding-top:1rem;border-top:1px solid var(--border);">{f}</div>"#))
            .unwrap_or_default();

        format!(
            r##"<div class="{cls}" style="{base_style}">
  {title}
  <div style="color:var(--text-muted);font-size:0.9375rem;line-height:1.6;">{body}</div>
  {footer}
</div>"##,
            cls = cls,
            base_style = base_style,
            title = title_html,
            body = self.body,
            footer = footer_html
        )
    }
}

// =====================================================================
// Badge — small status / count chip. 11 variants.
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    Default,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
    Brand,
    Cool,
    Warm,
    Purple,
    Outline,
}

impl BadgeKind {
    fn cls(self) -> &'static str {
        match self {
            BadgeKind::Default => "badge",
            BadgeKind::Primary => "badge badge-primary",
            BadgeKind::Success => "badge badge-success",
            BadgeKind::Warning => "badge badge-warning",
            BadgeKind::Danger => "badge badge-danger",
            BadgeKind::Info => "badge badge-info",
            BadgeKind::Brand => "badge badge-brand",
            BadgeKind::Cool => "badge badge-cool",
            BadgeKind::Warm => "badge badge-warm",
            BadgeKind::Purple => "badge badge-purple",
            BadgeKind::Outline => "badge badge-outline",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Badge {
    label: String,
    kind: BadgeKind,
    pill: bool,
    icon: Option<String>,
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: BadgeKind::Default,
            pill: false,
            icon: None,
        }
    }

    pub fn default(mut self) -> Self { self.kind = BadgeKind::Default; self }
    pub fn primary(mut self) -> Self { self.kind = BadgeKind::Primary; self }
    pub fn success(mut self) -> Self { self.kind = BadgeKind::Success; self }
    pub fn warning(mut self) -> Self { self.kind = BadgeKind::Warning; self }
    pub fn danger(mut self) -> Self { self.kind = BadgeKind::Danger; self }
    pub fn info(mut self) -> Self { self.kind = BadgeKind::Info; self }
    pub fn brand(mut self) -> Self { self.kind = BadgeKind::Brand; self }
    pub fn cool(mut self) -> Self { self.kind = BadgeKind::Cool; self }
    pub fn warm(mut self) -> Self { self.kind = BadgeKind::Warm; self }
    pub fn purple(mut self) -> Self { self.kind = BadgeKind::Purple; self }
    pub fn outline(mut self) -> Self { self.kind = BadgeKind::Outline; self }
    pub fn pill(mut self) -> Self { self.pill = true; self }
    pub fn icon(mut self, fa_name: impl Into<String>) -> Self { self.icon = Some(fa_name.into()); self }
    pub fn with_kind(mut self, k: BadgeKind) -> Self { self.kind = k; self }

    pub fn render(self) -> String {
        let cls = self.kind.cls();
        let radius = if self.pill { "border-radius:9999px;" } else { "" };
        let icon_html = self.icon.as_deref()
            .map(|i| format!(r#" <i data-lucide="{i}" style="font-size:0.75em;"></i>"#))
            .unwrap_or_default();
        format!(
            r##"<span class="{cls}" style="{radius}">{label}{icon}</span>"##,
            cls = cls,
            radius = radius,
            label = html_escape_text(&self.label),
            icon = icon_html
        )
    }
}

// =====================================================================
// Input — labelled form control. Variants: text, email, password, number,
// textarea, select.
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Text,
    Email,
    Password,
    Number,
    Url,
    Tel,
    Textarea,
    Select,
}

impl InputKind {
    fn tag(self) -> &'static str {
        match self {
            InputKind::Text | InputKind::Email | InputKind::Password | InputKind::Number | InputKind::Url | InputKind::Tel => "input",
            InputKind::Textarea => "textarea",
            InputKind::Select => "select",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    name: String,
    label: Option<String>,
    placeholder: Option<String>,
    value: String,
    kind: InputKind,
    required: bool,
    disabled: bool,
    icon: Option<String>,
    help: Option<String>,
    error: Option<String>,
    options: Vec<(String, String)>, // (value, label) for select
    rows: u32,
    extra_attrs: String,
}

impl Input {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: None,
            placeholder: None,
            value: String::new(),
            kind: InputKind::Text,
            required: false,
            disabled: false,
            icon: None,
            help: None,
            error: None,
            options: Vec::new(),
            rows: 4,
            extra_attrs: String::new(),
        }
    }

    pub fn text(mut self) -> Self { self.kind = InputKind::Text; self }
    pub fn email(mut self) -> Self { self.kind = InputKind::Email; self }
    pub fn password(mut self) -> Self { self.kind = InputKind::Password; self }
    pub fn number(mut self) -> Self { self.kind = InputKind::Number; self }
    pub fn url(mut self) -> Self { self.kind = InputKind::Url; self }
    pub fn tel(mut self) -> Self { self.kind = InputKind::Tel; self }
    pub fn textarea(mut self) -> Self { self.kind = InputKind::Textarea; self }
    pub fn select(mut self, options: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        self.kind = InputKind::Select;
        self.options = options.into_iter().map(|(v, l)| (v.into(), l.into())).collect();
        self
    }

    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self { self.placeholder = Some(p.into()); self }
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    pub fn required(mut self) -> Self { self.required = true; self }
    pub fn disabled(mut self) -> Self { self.disabled = true; self }
    pub fn icon(mut self, fa_name: impl Into<String>) -> Self { self.icon = Some(fa_name.into()); self }
    pub fn help(mut self, h: impl Into<String>) -> Self { self.help = Some(h.into()); self }
    pub fn error(mut self, e: impl Into<String>) -> Self { self.error = Some(e.into()); self }
    pub fn rows(mut self, r: u32) -> Self { self.rows = r; self }
    pub fn attr(mut self, k: &str, v: &str) -> Self {
        if !self.extra_attrs.is_empty() { self.extra_attrs.push(' '); }
        self.extra_attrs.push_str(&format!(r#"{}="{}""#, k, html_escape_attr(v)));
        self
    }

    pub fn render(self) -> String {
        let label_html = self.label.as_deref().map(|l| {
            format!(r#"<label for="{name}" class="label">{l}{req}</label>"#,
                name = self.name,
                l = html_escape_text(l),
                req = if self.required { r#" <span style="color:var(--epsx-red);">*</span>"# } else { "" }
            )
        }).unwrap_or_default();

        let help_html = self.help.as_deref()
            .map(|h| format!(r#"<p style="margin-top:0.375rem;font-size:0.8125rem;color:var(--text-subtle);">{h}</p>"#,
                h = html_escape_text(h)))
            .unwrap_or_default();

        let error_html = self.error.as_deref()
            .map(|e| format!(r#"<p style="margin-top:0.375rem;font-size:0.8125rem;color:var(--epsx-red);">{e}</p>"#,
                e = html_escape_text(e)))
            .unwrap_or_default();

        let icon_html = self.icon.as_deref().map(|i| {
            format!(r#"<i data-lucide="{i}" style="position:absolute;left:0.875rem;top:50%;transform:translateY(-50%);color:var(--text-subtle);pointer-events:none;"></i>"#)
        });

        let control = match self.kind {
            InputKind::Textarea => {
                let pad_left = if icon_html.is_some() { "padding-left:2.5rem;" } else { "" };
                format!(
                    r#"<textarea id="{id}" name="{id}" rows="{rows}" placeholder="{ph}" {req} {dis} {attrs} class="input" style="{pad_left}{style}">{val}</textarea>"#,
                    id = self.name,
                    rows = self.rows,
                    ph = self.placeholder.as_deref().unwrap_or(""),
                    req = if self.required { "required" } else { "" },
                    dis = if self.disabled { "disabled" } else { "" },
                    attrs = self.extra_attrs,
                    pad_left = pad_left,
                    style = if self.error.is_some() { "border-color:var(--epsx-red);" } else { "" },
                    val = html_escape_text(&self.value)
                )
            }
            InputKind::Select => {
                let opts: Vec<String> = self.options.iter().map(|(v, l)| {
                    let sel = if v == &self.value { " selected" } else { "" };
                    format!(r#"<option value="{v}"{sel}>{l}</option>"#,
                        v = html_escape_attr(v),
                        sel = sel,
                        l = html_escape_text(l))
                }).collect();
                format!(
                    r#"<select id="{id}" name="{id}" {req} {dis} {attrs} class="input">{opts}</select>"#,
                    id = self.name,
                    req = if self.required { "required" } else { "" },
                    dis = if self.disabled { "disabled" } else { "" },
                    attrs = self.extra_attrs,
                    opts = opts.join("")
                )
            }
            _ => {
                let pad_left = if icon_html.is_some() { "padding-left:2.5rem;" } else { "" };
                let input_type = match self.kind {
                    InputKind::Text => "text",
                    InputKind::Email => "email",
                    InputKind::Password => "password",
                    InputKind::Number => "number",
                    InputKind::Url => "url",
                    InputKind::Tel => "tel",
                    _ => "text",
                };
                format!(
                    r#"<input id="{id}" name="{id}" type="{ty}" placeholder="{ph}" value="{val}" {req} {dis} {attrs} class="input" style="{pad_left}{style}" />"#,
                    id = self.name,
                    ty = input_type,
                    ph = self.placeholder.as_deref().unwrap_or(""),
                    val = html_escape_attr(&self.value),
                    req = if self.required { "required" } else { "" },
                    dis = if self.disabled { "disabled" } else { "" },
                    attrs = self.extra_attrs,
                    pad_left = pad_left,
                    style = if self.error.is_some() { "border-color:var(--epsx-red);" } else { "" }
                )
            }
        };

        let field_html = if icon_html.is_some() {
            format!(r#"<div class="input-icon-wrap">{icon}{control}</div>"#,
                icon = icon_html.unwrap(),
                control = control)
        } else {
            control
        };

        format!(r#"<div style="margin-bottom:1rem;">{label}{field}{help}{error}</div>"#,
            label = label_html,
            field = field_html,
            help = help_html,
            error = error_html)
    }
}

// =====================================================================
// StatCard — compact metric tile (label + value + optional change).
// =====================================================================

#[derive(Debug, Clone)]
pub struct StatCard {
    label: String,
    value: String,
    change: Option<String>,
    change_kind: BadgeKind,
    icon: Option<String>,
    icon_color: String,
    href: Option<String>,
}

impl StatCard {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            change: None,
            change_kind: BadgeKind::Success,
            icon: None,
            icon_color: "var(--epsx-orange)".to_string(),
            href: None,
        }
    }

    pub fn change(mut self, c: impl Into<String>, kind: BadgeKind) -> Self {
        self.change = Some(c.into());
        self.change_kind = kind;
        self
    }
    pub fn icon(mut self, fa_name: impl Into<String>, color: impl Into<String>) -> Self {
        self.icon = Some(fa_name.into());
        self.icon_color = color.into();
        self
    }
    pub fn href(mut self, h: impl Into<String>) -> Self { self.href = Some(h.into()); self }

    pub fn render(self) -> String {
        let icon_html = self.icon.as_deref().map(|i| {
            format!(r#"<div style="width:2.75rem;height:2.75rem;border-radius:0.875rem;display:flex;align-items:center;justify-content:center;background:rgba(249,115,22,0.1);color:{c};font-size:1.25rem;flex-shrink:0;"><i data-lucide="{i}"></i></div>"#,
                c = self.icon_color, i = i)
        }).unwrap_or_default();

        let change_html = self.change.as_deref().map(|c| {
            let b = Badge::new(c).with_kind(self.change_kind).pill().icon(if c.starts_with('-') { "arrow-down" } else { "arrow-up" }).render();
            format!(r#"<div style="margin-top:0.5rem;">{b}</div>"#, b = b)
        }).unwrap_or_default();

        let card_body = format!(
            r##"<div class="card-glass hover-scale" style="padding:1.5rem;display:flex;align-items:flex-start;gap:1rem;">
  {icon}
  <div style="flex:1;min-width:0;">
    <p style="font-size:0.875rem;color:var(--text-muted);margin:0;font-weight:500;">{label}</p>
    <p style="font-size:1.75rem;font-weight:700;color:var(--text);margin:0.25rem 0 0;line-height:1.2;">{value}</p>
    {change}
  </div>
</div>"##,
            icon = icon_html,
            label = html_escape_text(&self.label),
            value = html_escape_text(&self.value),
            change = change_html
        );

        if let Some(href) = self.href {
            format!(r#"<a href="{href}" style="text-decoration:none;">{card_body}</a>"#,
                href = html_escape_attr(&href), card_body = card_body)
        } else {
            card_body
        }
    }
}

// =====================================================================
// Tabs — tabbed content switcher. Renders the tab bar; callers handle
// panel bodies with the `activateTab` global JS function.
// =====================================================================

#[derive(Debug, Clone)]
pub struct Tabs {
    group: String,
    tabs: Vec<(String, String)>, // (name, label)
    active: String,
}

impl Tabs {
    pub fn new(group: impl Into<String>) -> Self {
        Self {
            group: group.into(),
            tabs: Vec::new(),
            active: String::new(),
        }
    }
    pub fn tab(mut self, name: impl Into<String>, label: impl Into<String>) -> Self {
        let n = name.into();
        if self.active.is_empty() { self.active = n.clone(); }
        self.tabs.push((n, label.into()));
        self
    }
    pub fn active(mut self, name: impl Into<String>) -> Self {
        self.active = name.into();
        self
    }
    pub fn render(self) -> String {
        let items: Vec<String> = self.tabs.iter().map(|(n, l)| {
            let active_cls = if n == &self.active { " active" } else { "" };
            format!(
                r##"<button type="button" data-tab-group="{g}" data-tab-name="{n}" class="tab{active}" onclick="epsx.activateTab('{g}','{n}')">{l}</button>"##,
                g = self.group, n = html_escape_attr(n), active = active_cls, l = html_escape_text(l)
            )
        }).collect();
        format!(r#"<div class="tabs-nav" role="tablist">{items}</div>"#,
            items = items.join(""))
    }
}

// =====================================================================
// Skeleton — animated loading placeholder.
// =====================================================================

#[derive(Debug, Clone)]
pub struct Skeleton {
    width: String,
    height: String,
    rounded: bool,
    count: u32,
    gap: u32,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            width: "100%".to_string(),
            height: "1rem".to_string(),
            rounded: false,
            count: 1,
            gap: 8,
        }
    }
    pub fn w(mut self, w: impl Into<String>) -> Self { self.width = w.into(); self }
    pub fn h(mut self, h: impl Into<String>) -> Self { self.height = h.into(); self }
    pub fn rounded(mut self) -> Self { self.rounded = true; self }
    pub fn count(mut self, n: u32) -> Self { self.count = n; self }
    pub fn gap(mut self, g: u32) -> Self { self.gap = g; self }

    pub fn render(self) -> String {
        let radius = if self.rounded { "border-radius:9999px;" } else { "border-radius:0.375rem;" };
        let bar = format!(
            r##"<div class="skeleton" style="width:{w};height:{h};{radius}"></div>"##,
            w = self.width, h = self.height, radius = radius
        );
        if self.count <= 1 {
            bar
        } else {
            format!(r#"<div style="display:flex;flex-direction:column;gap:{g}px;">{bars}</div>"#,
                g = self.gap, bars = vec![bar; self.count as usize].join(""))
        }
    }
}

// =====================================================================
// Icon — single Lucide icon helper.
// =====================================================================

#[derive(Debug, Clone)]
pub struct Icon {
    name: String,
    size: Option<String>,
    color: Option<String>,
    extra_cls: String,
}

impl Icon {
    pub fn fa(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: None,
            color: None,
            extra_cls: String::new(),
        }
    }
    pub fn regular(mut self) -> Self { self }
    pub fn brand(mut self) -> Self { self }
    pub fn size(mut self, s: impl Into<String>) -> Self { self.size = Some(s.into()); self }
    pub fn color(mut self, c: impl Into<String>) -> Self { self.color = Some(c.into()); self }
    pub fn cls(mut self, c: impl Into<String>) -> Self { self.extra_cls = c.into(); self }

    pub fn render(self) -> String {
        let cls = if self.extra_cls.is_empty() { String::new() } else { format!(r#" class="{}""#, self.extra_cls) };
        let mut style = String::new();
        if let Some(s) = &self.size { style.push_str(&format!("width:{s};height:{s};")); }
        if let Some(c) = &self.color { style.push_str(&format!("color:{c};")); }
        let style_attr = if style.is_empty() { String::new() } else { format!(r#" style="{style}""#) };
        format!(r#"<i data-lucide="{name}"{cls}{style_attr}></i>"#, name = self.name, cls = cls, style_attr = style_attr)
    }
}

// =====================================================================
// Helpers — small pure functions used by the builders.
// =====================================================================

fn html_escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// =====================================================================
// Convenience constructors for the most common usages.
// =====================================================================

/// `btn_primary(label)` → `Btn::new(label).primary().render()`
pub fn btn_primary(label: impl Into<String>) -> String { Btn::new(label).primary().render() }
/// `btn_gradient(label)` → `Btn::new(label).gradient().render()`
pub fn btn_gradient(label: impl Into<String>) -> String { Btn::new(label).gradient().render() }
/// `btn_brand(label)` → `Btn::new(label).brand().render()`
pub fn btn_brand(label: impl Into<String>) -> String { Btn::new(label).brand().render() }
/// `btn_outline(label)` → `Btn::new(label).outline().render()`
pub fn btn_outline(label: impl Into<String>) -> String { Btn::new(label).outline().render() }
/// `btn_ghost(label)` → `Btn::new(label).ghost().render()`
pub fn btn_ghost(label: impl Into<String>) -> String { Btn::new(label).ghost().render() }
/// `btn_danger(label)` → `Btn::new(label).danger().render()`
pub fn btn_danger(label: impl Into<String>) -> String { Btn::new(label).danger().render() }

/// `card(title, body)` → `Card::new(title, body).glass().render()`
pub fn card(title: impl Into<String>, body: impl Into<String>) -> String {
    Card::new(title, body).glass().render()
}
/// `card_hover(title, body)` → glass card with hover scale
pub fn card_hover(title: impl Into<String>, body: impl Into<String>) -> String {
    Card::new(title, body).glass().hover().render()
}
/// `card_insight(title, body)` → insight card
pub fn card_insight(title: impl Into<String>, body: impl Into<String>) -> String {
    Card::new(title, body).insight().render()
}

/// `badge_success(label)` → `Badge::new(label).success().render()`
pub fn badge_success(label: impl Into<String>) -> String { Badge::new(label).success().render() }
/// `badge_danger(label)` → `Badge::new(label).danger().render()`
pub fn badge_danger(label: impl Into<String>) -> String { Badge::new(label).danger().render() }
/// `badge_warn(label)` → `Badge::new(label).warning().render()`
pub fn badge_warn(label: impl Into<String>) -> String { Badge::new(label).warning().render() }
/// `badge_info(label)` → `Badge::new(label).info().render()`
pub fn badge_info(label: impl Into<String>) -> String { Badge::new(label).info().render() }
/// `badge_warm(label)` → `Badge::new(label).warm().render()`
pub fn badge_warm(label: impl Into<String>) -> String { Badge::new(label).warm().render() }

/// `stat(label, value)` → `StatCard::new(label, value).render()`
pub fn stat(label: impl Into<String>, value: impl Into<String>) -> String {
    StatCard::new(label, value).render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btn_primary_renders() {
        let html = Btn::new("Go").primary().render();
        assert!(html.contains("btn-primary"));
        assert!(html.contains("Go"));
        assert!(html.contains("<button"));
    }

    #[test]
    fn btn_gradient_with_icons() {
        let html = Btn::new("Trade").gradient().lg().icon_left("bolt").icon_right("arrow-right").render();
        assert!(html.contains("btn-gradient"));
        assert!(html.contains("btn-lg"));
        assert!(html.contains(r#"data-lucide="bolt""#));
        assert!(html.contains(r#"data-lucide="arrow-right""#));
    }

    #[test]
    fn btn_as_link() {
        let html = Btn::new("Docs").outline().href("/docs").render();
        assert!(html.starts_with("<a "));
        assert!(html.contains(r#"href="/docs""#));
    }

    #[test]
    fn card_with_hover_and_icon() {
        let html = Card::new("Hello", "World").glass().hover().icon("user").render();
        assert!(html.contains("card-glass"));
        assert!(html.contains("hover-scale"));
        assert!(html.contains(r#"data-lucide="user""#));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn card_body_only() {
        let html = Card::body_only("just a body").render();
        assert!(!html.contains("<h3"));
        assert!(html.contains("just a body"));
    }

    #[test]
    fn badge_variants() {
        assert!(Badge::new("ok").success().pill().render().contains("badge-success"));
        assert!(Badge::new("x").danger().render().contains("badge-danger"));
        assert!(Badge::new("!").warning().icon("exclamation").render().contains(r#"data-lucide="exclamation""#));
    }

    #[test]
    fn input_text_with_label() {
        let html = Input::new("email").email().label("Email").required().placeholder("you@x.com").render();
        assert!(html.contains(r#"type="email""#));
        assert!(html.contains(r#"<label for="email""#));
        assert!(html.contains("required"));
        assert!(html.contains(r#"placeholder="you@x.com""#));
    }

    #[test]
    fn input_with_icon() {
        let html = Input::new("user").text().icon("user").label("Username").render();
        assert!(html.contains("input-icon-wrap"));
        assert!(html.contains(r#"data-lucide="user""#));
    }

    #[test]
    fn input_select() {
        let opts = vec![("us", "United States"), ("ca", "Canada")];
        let html = Input::new("country").label("Country").select(opts).value("ca").render();
        assert!(html.contains("<select"));
        assert!(html.contains(r#"value="us""#));
        assert!(html.contains("selected"));
    }

    #[test]
    fn input_textarea() {
        let html = Input::new("msg").textarea().rows(6).label("Message").value("hi").render();
        assert!(html.contains("<textarea"));
        assert!(html.contains(r#"rows="6""#));
        assert!(html.contains(">hi</textarea>"));
    }

    #[test]
    fn stat_card_with_change() {
        let html = StatCard::new("Revenue", "$1,234").change("+12.5%", BadgeKind::Success).icon("chart-line", "var(--epsx-orange)").render();
        assert!(html.contains("Revenue"));
        assert!(html.contains("$1,234"));
        assert!(html.contains(r#"data-lucide="chart-line""#));
        assert!(html.contains("+12.5%"));
    }

    #[test]
    fn tabs_render_all_with_active() {
        let html = Tabs::new("g1").tab("a", "Alpha").tab("b", "Beta").tab("c", "Gamma").active("b").render();
        assert!(html.contains(r#"data-tab-group="g1""#));
        assert!(html.contains(r#"data-tab-name="b" class="tab active""#));
        assert!(html.contains("Alpha"));
        assert!(html.contains("Gamma"));
    }

    #[test]
    fn skeleton_single() {
        let html = Skeleton::new().w("200px").h("20px").render();
        assert!(html.contains("width:200px"));
        assert!(html.contains("height:20px"));
    }

    #[test]
    fn skeleton_count() {
        let html = Skeleton::new().count(3).gap(12).render();
        assert_eq!(html.matches(r#"class="skeleton""#).count(), 3);
        assert!(html.contains("gap:12px"));
    }

    #[test]
    fn icon_with_size_and_color() {
        let html = Icon::fa("user").size("1.5rem").color("var(--epsx-orange)").render();
        assert!(html.contains(r#"data-lucide="user""#));
        assert!(html.contains("width:1.5rem"));
        assert!(html.contains("height:1.5rem"));
        assert!(html.contains("color:var(--epsx-orange)"));
    }

    #[test]
    fn convenience_helpers() {
        assert!(btn_primary("X").contains("btn-primary"));
        assert!(btn_gradient("X").contains("btn-gradient"));
        assert!(btn_danger("X").contains("btn-danger"));
        assert!(card("t", "b").contains("card-glass"));
        assert!(card_hover("t", "b").contains("hover-scale"));
        assert!(badge_success("ok").contains("badge-success"));
        assert!(badge_warn("!").contains("badge-warning"));
        assert!(stat("k", "v").contains("card-glass"));
    }

    #[test]
    fn html_escapes_attrs() {
        let html = Btn::new("ok").attr("data-x", r#"a"b&c<d>"#).render();
        assert!(!html.contains(r#"data-x="a"b&c<d>""#));
        assert!(html.contains("a&quot;b&amp;c&lt;d&gt;"));
    }
}
