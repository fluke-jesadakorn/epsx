//! Public service introduction. Deliberately independent of PayProvider and sessions.
use super::types::Environment;
use dioxus::prelude::*;

// Keep the head stylesheet mounted above the router. Dioxus head styles are
// immutable; theme and route updates must not replace their children.
#[component]
pub(super) fn LandingStyles() -> Element {
    rsx! { document::Style { {include_str!("home.css")} } }
}

#[component]
pub(super) fn Landing(environment: Environment, mut dark: Signal<bool>) -> Element {
    let mut menu_open = use_signal(|| false);
    let dashboard = format!("/dashboard?environment={}", environment.as_str());
    let docs = format!("/docs/merchant?environment={}", environment.as_str());
    let api = format!("/docs?environment={}", environment.as_str());
    rsx! {
        document::Title { "EPSX Pay · Crypto payments, simply." }
        document::Meta {
            name: "description",
            content: "Accept USDT and USDC with EPSX Pay. Create payment links, sell packages, and track crypto payments with hosted checkout, APIs, and webhooks."
        }
        div { class: if dark() { "epsx-merchant epsx-pay-home dark" } else { "epsx-merchant epsx-pay-home" },
            a { class: "ph-skip", href: "#pay-home-main", "Skip to content" }
            header { class: "ph-header",
                div { class: "ph-wrap ph-header-inner",
                    Link { class: "ph-brand", to: format!("/?environment={}", environment.as_str()), "aria-label": "EPSX Pay home",
                        img { src: "/brand-icon.svg", alt: "", width: "32", height: "32" }
                        "EPSX" span { "Pay" }
                    }
                    nav { id: "pay-home-nav", class: if menu_open() { "ph-nav ph-nav-open" } else { "ph-nav" }, "aria-label": "Main navigation",
                        for (href, label) in [("#features", "Features"), ("#how-it-works", "How it works"), ("#pricing", "Pricing")] {
                            a { href, onclick: move |_| menu_open.set(false), "{label}" }
                        }
                        Link { to: docs.clone(), "Developer guide" }
                    }
                    div { class: "ph-header-actions",
                        button { class: "ph-icon-button", "aria-label": "Toggle theme", "aria-pressed": dark().to_string(), onclick: move |_| dark.toggle(),
                            Icon { kind: if dark() { "sun" } else { "moon" } }
                        }
                        Link { class: "ph-button ph-button-small", to: dashboard.clone(), "Dashboard" Icon { kind: "arrow" } }
                        button { class: "ph-icon-button ph-menu-button", "aria-label": "Toggle navigation", "aria-controls": "pay-home-nav", "aria-expanded": menu_open().to_string(), onclick: move |_| menu_open.toggle(),
                            Icon { kind: if menu_open() { "close" } else { "menu" } }
                        }
                    }
                }
            }
            main { id: "pay-home-main", tabindex: "-1",
                section { class: "ph-wrap ph-hero", "aria-labelledby": "pay-home-title",
                    div { class: "ph-hero-copy",
                        p { class: "ph-eyebrow", span { class: "ph-dot" } "YOUR BUSINESS. YOUR WALLET." }
                        h1 { id: "pay-home-title", "Crypto payments," br {} em { "simply." } }
                        p { class: "ph-lead", "Accept USDT and USDC with a link. Sell your packages, share a checkout, and follow every payment — all in one place." }
                        div { class: "ph-actions",
                            Link { class: "ph-button", to: dashboard.clone(), "Start accepting payments" Icon { kind: "arrow" } }
                            Link { class: "ph-text-link", to: docs.clone(), "Developer guide" Icon { kind: "arrow-up" } }
                        }
                        p { class: "ph-hero-note", Icon { kind: "check" } "No monthly charge" span { "·" } "Wallet-based sign in" }
                        div { class: "ph-tokens", span { "Built for stablecoins" } span { class: "ph-token", b { class: "ph-usdt", "₮" } "USDT" } span { class: "ph-token", b { class: "ph-usdc", "$" } "USDC" } }
                    }
                    CheckoutPreview {}
                }
                section { id: "features", class: "ph-features", "aria-labelledby": "ph-features-title",
                    div { class: "ph-wrap",
                        div { class: "ph-section-heading",
                            div { p { class: "ph-eyebrow", "LESS SETUP. MORE POSSIBILITY." } h2 { id: "ph-features-title", "From your first link" br {} "to your own integration." } }
                            p { "Start with a checkout you can share. Add the tools your business needs as you grow." }
                        }
                        div { class: "ph-feature-grid",
                            for (kind, title, body) in [
                                ("link", "A link. Ready to share.", "Create a payment link with an amount, an expiry, and an optional usage limit. Send it wherever your customers are."),
                                ("package", "Your packages, on display.", "Give your products and services a home with a shareable storefront and prices in USDT or USDC."),
                                ("scan", "A familiar way to pay.", "Customers can scan a checkout QR code or connect a compatible wallet. Every checkout has its own payment details."),
                                ("activity", "Follow every payment.", "See confirmed payments, track funds ready for collection, and manage refunds from your merchant workspace.")
                            ] {
                                article { class: "ph-feature",
                                    span { class: "ph-feature-icon", Icon { kind } }
                                    h3 { "{title}" } p { "{body}" }
                                }
                            }
                        }
                        article { class: "ph-integration",
                            div { class: "ph-integration-symbol", Icon { kind: "code" } }
                            div { h3 { "Your checkout. Connected to your business." } p { "Create checkouts with the API and receive signed payment events through webhooks. Keep your own order and service workflows in sync." } }
                            Link { class: "ph-text-link", to: api.clone(), "Explore the API" Icon { kind: "arrow-up" } }
                        }
                    }
                }
                section { id: "how-it-works", class: "ph-wrap ph-section", "aria-labelledby": "ph-steps-title",
                    div { class: "ph-section-heading", div { p { class: "ph-eyebrow", "A SIMPLE START" } h2 { id: "ph-steps-title", "Open for business in three steps." } } }
                    ol { class: "ph-steps",
                        for (number, title, body) in [
                            ("01", "Make it yours", "Connect MetaMask, sign in with your wallet, and choose a name for your shop."),
                            ("02", "Create something to sell", "Set up a package or a payment link. Choose your token, amount, and checkout details."),
                            ("03", "Share. Track. Collect.", "Send your checkout to customers, follow confirmed payments, and collect funds to your merchant wallet.")
                        ] {
                            li { span { class: "ph-step-number", "{number}" } h3 { "{title}" } p { "{body}" } }
                        }
                    }
                }
                section { id: "pricing", class: "ph-pricing-section", "aria-labelledby": "ph-pricing-title",
                    div { class: "ph-wrap ph-pricing-layout",
                        div { class: "ph-pricing-copy", p { class: "ph-eyebrow", "STRAIGHTFORWARD PRICING" } h2 { id: "ph-pricing-title", "No monthly bill." br {} "Just payment fees." } p { "Your shop, payment links, dashboard, API, and outgoing webhooks are included." } p { class: "ph-fine-print", "Network gas is charged separately. QR payment fees apply when funds are collected. Escrow fees apply only when funds are released." } }
                        div { class: "ph-prices",
                            article { class: "ph-price-card", span { class: "ph-price-label", "DIRECT PAYMENTS" } p { class: "ph-price", "0.5" span { "%" } } h3 { "Simple payments for everyday business." } p { "Accept payments through links and hosted checkout, then track collection in your dashboard." } Link { class: "ph-text-link", to: dashboard.clone(), "Get started" Icon { kind: "arrow" } } }
                            article { class: "ph-price-card", span { class: "ph-price-label", "ESCROW PAYMENTS" } p { class: "ph-price", "1" span { "%" } } h3 { "Hold funds until it’s time to release." } p { "Use escrow for payments that need a release step. The processing fee applies on release." } Link { class: "ph-text-link", to: api.clone(), "Read the guide" Icon { kind: "arrow" } } }
                        }
                    }
                }
                section { class: "ph-wrap ph-closing", "aria-labelledby": "ph-closing-title",
                    div { p { class: "ph-eyebrow", "YOUR NEXT CHAPTER" } h2 { id: "ph-closing-title", "Let’s get your business paid." } p { "A wallet, a shop name, and something worth sharing." } }
                    Link { class: "ph-button", to: dashboard.clone(), "Start accepting payments" Icon { kind: "arrow" } }
                }
            }
            footer { class: "ph-footer", div { class: "ph-wrap ph-footer-inner",
                div { span { class: "ph-footer-brand", "EPSX Pay" } p { "Crypto payments, simply." } }
                nav { "aria-label": "Footer navigation", Link { to: dashboard, "Dashboard" } Link { to: docs, "Developer guide" } Link { to: api, "API reference" } }
            } }
        }
    }
}

#[component]
fn CheckoutPreview() -> Element {
    rsx! {
        figure { class: "ph-preview", "aria-label": "Sample checkout preview",
            div { class: "ph-preview-grid", "aria-hidden": "true" }
            div { class: "ph-checkout", "aria-hidden": "true",
                div { class: "ph-checkout-header", span { class: "ph-sample-logo", Icon { kind: "package" } } div { strong { "Studio North" } small { "A little creativity, delivered." } } span { class: "ph-checkout-mark", Icon { kind: "lock" } } }
                div { class: "ph-checkout-item", span { "Design resource pack" } strong { "49.00" small { "USDC" } } p { "One-time payment" } }
                div { class: "ph-preview-methods", span { class: "ph-method-active", Icon { kind: "scan" } "QR / Transfer" } span { Icon { kind: "wallet" } "Connect wallet" } }
                div { class: "ph-payment-illustration",
                    svg { view_box: "0 0 100 100", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                        rect { width: "100", height: "100", rx: "10", fill: "white" }
                        path { d: "M12 12h24v24H12z M64 12h24v24H64z M12 64h24v24H12z", stroke: "#453775", stroke_width: "5" }
                        path { d: "M20 20h8v8h-8z M72 20h8v8h-8z M20 72h8v8h-8z M44 12h8v16h-8z M44 36h12v8H44z M12 44h16v8H12z M32 48h8v12h-8z M48 56h8v12h-8z M64 44h8v8h-8z M80 44h8v16h-8z M64 64h16v8H64z M84 72h4v16h-4z M44 80h12v8H44z M60 80h12v8H60z", fill: "#453775" }
                        rect { x: "39", y: "39", width: "22", height: "22", rx: "7", fill: "#7855df" }
                        path { d: "M44 50h12m-6-6v12", stroke: "white", stroke_width: "2.5", stroke_linecap: "round" }
                    }
                    p { "Scan. Send. You’re all set." }
                }
                div { class: "ph-checkout-bottom", span { "Powered by" } strong { "EPSX Pay" } }
            }
            div { class: "ph-preview-receipt", "aria-hidden": "true", span { class: "ph-receipt-icon", Icon { kind: "check" } } div { strong { "Payment confirmed" } small { "One more happy customer." } } span { class: "ph-receipt-amount", "+49.00" small { "USDC" } } }
            figcaption { span { class: "ph-dot" } "CHECKOUT PREVIEW · SAMPLE ONLY" }
        }
    }
}

#[component]
fn Icon(kind: String) -> Element {
    let path = match kind.as_str() {
        "arrow" => "M4 12h16m-6-6 6 6-6 6",
        "arrow-up" => "M6 18 18 6M6 6h12v12",
        "check" => "m5 12 4 4L19 6",
        "sun" => "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5",
        "moon" => "M20.5 13.5A9 9 0 0 1 10.5 3a9 9 0 1 0 10 10.5Z",
        "menu" => "M4 6h16M4 12h16M4 18h16",
        "close" => "m6 6 12 12M6 18 18 6",
        "link" => "m10 13 4-4m-5 7-2 2a4 4 0 0 1-6-6l4-4a4 4 0 0 1 6 0m2 0 2-2a4 4 0 0 1 6 6l-4 4a4 4 0 0 1-6 0",
        "package" => "m3 7 9-5 9 5v10l-9 5-9-5V7Zm0 0 9 5 9-5M12 12v10M7.5 4.5l9 5V14",
        "scan" => "M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5M7 7h3v3H7zm7 0h3v3h-3zM7 14h3v3H7zm7 0h3v3h-3z",
        "activity" => "M3 3v18h18M6 15l4-5 4 3 6-8",
        "code" => "m8 6-6 6 6 6m8-12 6 6-6 6M14 3l-4 18",
        "lock" => "M7 10V7a5 5 0 0 1 10 0v3M5 10h14v11H5zM12 14v3",
        "wallet" => "M20 8V5H4a2 2 0 0 1 0-4h14v4M4 5a2 2 0 0 0-2 2v13h20V8H5m17 4h-7v4h7",
        _ => "",
    };
    rsx! { svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.7", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", "focusable": "false", path { d: path } } }
}
