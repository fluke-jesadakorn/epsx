use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Terms of service");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content prose",
                h1 { "Terms of service" }
                p { "By using EPSX you agree to these terms." }
                h2 { "Wallets" }
                p { "You are responsible for your wallet. We do not custody your funds." }
                h2 { "Payments" }
                p { "Payments are settled on-chain. Refunds are subject to the merchant's policy." }
                h2 { "Subscriptions" }
                p { "Cancel anytime. Plans renew automatically at the end of each billing period unless cancelled." }
                h2 { "Liability" }
                p { "EPSX is provided as-is. We are not liable for losses arising from market movements or smart-contract bugs." }
                h2 { "Contact" }
                p { "info@epsx.io" }
            }
        }
    })
}
