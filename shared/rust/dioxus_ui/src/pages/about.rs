use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("About");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            section { class: "about-section container page-content",
                div { class: "about-hero",
                    h1 { class: "section-title", "About EPSX" }
                    p { class: "text-lg text-muted-foreground mt-4",
                        "EPSX is a production-grade Web3 commerce platform: visual page builder, on-chain payments, programmable subscriptions, and paymaster-sponsored gas — all running as Rust microservices on BSC."
                    }
                }
                div { class: "about-mvv mt-12 grid grid-cols-1 md:grid-cols-2 gap-6",
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Mission" } }
                        div { class: "card-body", p { "Make on-chain commerce as easy to ship as a no-code site, with the same performance and auditability of native smart contracts." } }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Vision" } }
                        div { class: "card-body", p { "An open Web3 commerce stack where every merchant, builder, and analyst can compose their own marketplace in days, not months." } }
                    }
                }
                div { class: "mt-12",
                    h2 { class: "section-title", "DataTech Platform" }
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mt-4",
                        div { class: "card card-glass",
                            div { class: "card-body",
                                h4 { class: "card-title", "Collection" }
                                p { class: "text-muted-foreground text-sm", "Stream on-chain events and market data in real time." }
                            }
                        }
                        div { class: "card card-glass",
                            div { class: "card-body",
                                h4 { class: "card-title", "Storage" }
                                p { class: "text-muted-foreground text-sm", "PostgreSQL + Redis for hot paths, cold storage for archives." }
                            }
                        }
                        div { class: "card card-glass",
                            div { class: "card-body",
                                h4 { class: "card-title", "Processing" }
                                p { class: "text-muted-foreground text-sm", "Rust services handle 8.5M+ rankings with sub-millisecond latency." }
                            }
                        }
                        div { class: "card card-glass",
                            div { class: "card-body",
                                h4 { class: "card-title", "Visualization" }
                                p { class: "text-muted-foreground text-sm", "Dioxus-powered dashboards with glassmorphism + gradient design." }
                            }
                        }
                        div { class: "card card-glass",
                            div { class: "card-body",
                                h4 { class: "card-title", "Machine learning" }
                                p { class: "text-muted-foreground text-sm", "Anomaly detection on payment flows, plan abandonment, churn signals." }
                            }
                        }
                        div { class: "card card-glass",
                            div { class: "card-body",
                                h4 { class: "card-title", "Governance" }
                                p { class: "text-muted-foreground text-sm", "Role-based access control at the backend, not the UI." }
                            }
                        }
                    }
                }
            }
        }
    })
}
