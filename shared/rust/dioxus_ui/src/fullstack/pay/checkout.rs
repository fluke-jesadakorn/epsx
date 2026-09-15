//! Checkout presentation. Payment proof and access decisions come from services.
use super::super::types::Action;
use super::super::{act_pay, types::*};
use super::{
    pairing_qr, transaction_explorer_url, wallet, Controller, CopyField, Expiry, OperationButtons,
};
use dioxus::prelude::*;

pub(super) fn valid_hash(hash: &str) -> bool {
    hash.len() == 66 && hash.starts_with("0x") && hash[2..].bytes().all(|b| b.is_ascii_hexdigit())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Loading,
    Ready,
    Wallet,
    Confirming,
    Activating,
    Success,
    Expired,
    Refunded,
    Escrow,
    Review,
}
fn stage(data: Option<&PageData>, session: &CheckoutSession, wallet: WalletPhase) -> Stage {
    let Some(data) = data else {
        return if session.pending.is_some() {
            Stage::Confirming
        } else {
            Stage::Loading
        };
    };
    let Some(p) = &data.payment else {
        return if session.pending.is_some() {
            Stage::Confirming
        } else {
            Stage::Loading
        };
    };
    match p.status.as_str() {
        "succeeded"
            if data.completion.as_ref().is_some_and(|c| {
                matches!(
                    c.fulfillment_status.as_str(),
                    "revoked" | "manual_override" | "expired"
                )
            }) =>
        {
            Stage::Review
        }
        "succeeded"
            if data.completion.as_ref().is_some_and(|c| !c.ready())
                || !data.completion_available =>
        {
            Stage::Activating
        }
        "succeeded" => Stage::Success,
        "refunded" => Stage::Refunded,
        "funded" | "disputed" => Stage::Escrow,
        "expired" if session.pending.is_none() => Stage::Expired,
        "verification_required" => Stage::Review,
        "failed" => Stage::Review,
        "confirming" | "processing" | "deposited" => Stage::Confirming,
        _ if session.pending.is_some() || wallet == WalletPhase::Submitted => Stage::Confirming,
        _ if wallet != WalletPhase::Idle => Stage::Wallet,
        _ => Stage::Ready,
    }
}
fn network(chain: u64) -> String {
    match chain {
        56 => "BNB Smart Chain".into(),
        97 => "BNB Smart Chain Testnet".into(),
        31337 => "Local test network".into(),
        _ => format!("Chain {chain}"),
    }
}
fn redirect_ready(data: Option<&PageData>, session: &CheckoutSession) -> bool {
    session.started
        && !session.redirected
        && data.is_some_and(|d| {
            d.completion_available
                && d.payment.as_ref().is_some_and(|p| p.status == "succeeded")
                && d.completion
                    .as_ref()
                    .is_some_and(|c| c.ready() && c.valid_return(&d.frontend_origin))
        })
}
async fn persist(c: Controller, id: &str) {
    let value = c.session.peek().clone();
    let _ = wallet::checkout_session(id, Some(value)).await;
}
async fn return_now(mut c: Controller) {
    let data = c.data.peek().clone();
    let Some(data) = data else {
        return;
    };
    let (Some(payment), Some(completion)) = (&data.payment, &data.completion) else {
        return;
    };
    if !completion.valid_return(&data.frontend_origin) {
        return;
    }
    c.session.write().redirected = true;
    if let Err(error) = wallet::return_purchase(
        &payment.checkout_id,
        &completion.return_url,
        &data.frontend_origin,
    )
    .await
    {
        c.message.set(error);
    }
}

fn pay(mut c: Controller, payment: Payment, selected_address: String, walletconnect: bool) {
    if *c.busy.peek() || c.session.peek().pending.is_some() {
        return;
    }
    c.busy.set(true);
    c.message.set(String::new());
    c.wallet_phase.set(WalletPhase::Connecting);
    spawn(async move {
        let result: Result<(), String> = async {
            let payer = if selected_address.is_empty() {
                wallet::connect(payment.chain_id, walletconnect).await?
            } else {
                selected_address
            };
            c.wallet_phase.set(WalletPhase::Preparing);
            let transfer = payment.payment_method == "transfer";
            let kind = if payment.mode == "escrow" {
                OperationKind::Deposit
            } else {
                OperationKind::Pay
            };
            let prepare = if transfer {
                Action::PrepareTransfer {
                    id: payment.checkout_id.clone(),
                    payer,
                }
            } else {
                Action::PrepareOperation {
                    id: payment.checkout_id.clone(),
                    checkout: true,
                    kind,
                    payer,
                }
            };
            let context = serde_json::to_string(&((c.credentials)().environment, &prepare))
                .map_err(|e| e.to_string())?;
            let prepared = c.perform(prepare).await?;
            if prepared.payment.as_ref().is_some_and(Payment::terminal) {
                return Err("This checkout has changed. Checking its latest status…".into());
            }
            let pending = PendingCheckout {
                checkout_id: payment.checkout_id.clone(),
                chain_id: payment.chain_id,
                hash: String::new(),
                operation_id: if transfer {
                    None
                } else {
                    Some(prepared.id.clone())
                },
                request_context: context,
            };
            let storage = if transfer {
                format!("epsx.checkout.transfer.{}", payment.checkout_id)
            } else {
                format!("epsx.merchant.tx.{}", prepared.id)
            };
            let hash = wallet::send_checkout(
                prepared
                    .transaction_parameters
                    .ok_or("Payment is temporarily unavailable. Please try again.")?,
                storage,
                prepared.approval_transaction,
                pending.clone(),
                move |phase| c.wallet_phase.set(phase),
            )
            .await?;
            let pending = PendingCheckout { hash, ..pending };
            c.session.write().started = true;
            c.session.write().pending = Some(pending.clone());
            persist(c, &payment.checkout_id).await;
            if let Some(id) = &pending.operation_id {
                c.perform(Action::ConfirmOperation {
                    id: id.clone(),
                    tx_hash: pending.hash.clone(),
                })
                .await?;
            }
            Ok(())
        }
        .await;
        if let Err(error) = result {
            c.message.set(error);
        }
        // The adapter stores the submitted hash before returning, so a failed
        // confirm request or a lost UI response can resume the same operation.
        if let Ok(session) = wallet::checkout_session(&payment.checkout_id, None).await {
            if session.pending.is_some() {
                c.session.set(session);
            }
        }
        if c.session.peek().pending.is_none() {
            c.wallet_phase.set(WalletPhase::Idle);
        }
        c.busy.set(false);
        c.revision += 1;
    });
}

#[component]
pub(super) fn Checkout(mut dark: Signal<bool>) -> Element {
    let mut c = use_context::<Controller>();
    let mut walletconnect = use_signal(|| false);
    let mut wallet_method = use_signal(|| false);
    let mut countdown = use_signal(|| None::<u8>);
    let mut waiting_seconds = use_signal(|| 0_u32);
    let mut pairing = use_signal(String::new);
    // These futures are owned by this route and cancelled on unmount.
    use_future(move || async move {
        loop {
            if wallet::second().await.is_err() {
                break;
            }
            let current = c.data.peek().clone();
            let session = c.session.peek().clone();
            let phase = stage(current.as_ref(), &session, *c.wallet_phase.peek());
            if matches!(phase, Stage::Confirming | Stage::Activating | Stage::Review) {
                waiting_seconds += 1;
            } else {
                waiting_seconds.set(0);
            }
            if *c.session_loaded.peek() && redirect_ready(current.as_ref(), &session) {
                let next = countdown.peek().map_or(3, |n| n.saturating_sub(1));
                countdown.set(Some(next));
                if next == 0 {
                    return_now(c).await;
                    break;
                }
            } else {
                countdown.set(None);
            }
            if *c.wallet_phase.peek() == WalletPhase::Connecting && *walletconnect.peek() {
                if let Ok(uri) = wallet::pairing().await {
                    pairing.set(uri);
                }
            }
        }
    });
    use_future(move || async move {
        loop {
            if wallet::pause().await.is_err() {
                break;
            }
            if *c.busy.peek() {
                continue;
            }
            let pending = c.session.peek().pending.clone();
            let Some(pending) = pending else {
                continue;
            };
            let Some(id) = &pending.operation_id else {
                continue;
            };
            let credentials = c.credentials.peek().clone();
            let key = match wallet::key(&format!("checkout-resume-{id}")).await {
                Ok(k) => k,
                Err(_) => continue,
            };
            let result = act_pay(
                Action::ReadOperation { id: id.clone() },
                credentials.clone(),
                key.clone(),
            )
            .await;
            match result {
                Ok(Ok(state)) if state.status == "failed" => {
                    c.session.write().pending = None;
                    c.wallet_phase.set(WalletPhase::Idle);
                    c.message
                        .set("The transaction failed. You can try the payment again.".into());
                    let _ = wallet::complete(&pending.request_context).await;
                    persist(c, &pending.checkout_id).await;
                }
                Ok(Ok(state)) if state.status == "confirmed" => {
                    c.revision += 1;
                }
                Ok(Ok(_)) => {
                    // Repeating the same hash is idempotent; no wallet send occurs.
                    let _ = act_pay(
                        Action::ConfirmOperation {
                            id: id.clone(),
                            tx_hash: pending.hash,
                        },
                        credentials,
                        key,
                    )
                    .await;
                }
                _ => {}
            }
        }
    });
    let data = (c.data)();
    let session = (c.session)();
    let phase = stage(data.as_ref(), &session, (c.wallet_phase)());
    let payment = data.as_ref().and_then(|d| d.payment.clone());
    let completion = data.as_ref().and_then(|d| d.completion.clone());
    let origin = data
        .as_ref()
        .map(|d| d.frontend_origin.clone())
        .unwrap_or_default();
    let message = (c.message)();
    let transaction = (c.transaction)().or_else(|| {
        payment
            .as_ref()
            .and_then(|p| p.tx_hash.clone().map(|h| (p.chain_id, h)))
            .or_else(|| {
                session
                    .pending
                    .as_ref()
                    .map(|p| (p.chain_id, p.hash.clone()))
            })
    });
    let total = payment
        .as_ref()
        .map(|p| {
            format!(
                "{} {}",
                display_amount(&p.amount, p.token_decimals.unwrap_or(0)),
                p.token
            )
        })
        .unwrap_or_default();
    let epsx = completion.is_some()
        || payment
            .as_ref()
            .is_some_and(|p| p.checkout_snapshot.kind == "epsx_plan");
    let step = match phase {
        Stage::Success => 3,
        Stage::Activating | Stage::Escrow => 2,
        Stage::Confirming | Stage::Review => 1,
        _ => 0,
    };
    rsx! {
        document::Title { "Complete your payment · EPSX Pay" }
        div { class:"pc-checkout",
            header { class:"pc-header",
                crate::navigation::AppLink { class:"pc-brand",href:"/",img{src:"/brand-icon.svg",alt:"",width:32,height:32} "EPSX Pay" }
                div { class:"flex items-center gap-4",
                    if !epsx && payment.is_some() {if let Some(p)=payment.clone(){Link{class:"pc-back",to:format!("/m/{}?environment={}",p.merchant_id,p.environment.as_str()),"← Back to merchant"}}}
                    else {crate::navigation::AppLink { class:"pc-back",href:format!("{origin}/plans"), "← Back to plans" }}
                    button { class:"pc-theme",r#type:"button",aria_label:"Toggle theme",aria_pressed:dark(),onclick:move |_|dark.toggle(),
                        crate::primitives::Icon {name:if dark(){"sun"}else{"moon"},size:20}
                    }
                }
            }
            main { class:"pc-main", "data-checkout-stage":format!("{phase:?}").to_ascii_lowercase(),
                if let Some(p) = payment.clone() {
                    section { class:"pc-summary",
                        p { class:"pc-eyebrow", "{p.checkout_snapshot.merchant_name}" }
                        h1 { class:"pc-title", "Complete your payment" }
                        p { class:"pc-description", "Your purchase, one simple checkout." }
                        div { class:"pc-order",
                            span { class:"pc-order-icon",crate::primitives::Icon{name:"arrow-up-right",size:24} }
                            div { h2 { class:"text-lg font-semibold", "{p.description}" }
                                if let Some(days)=p.checkout_snapshot.duration_days { p {class:"pc-muted","{days} days of access"} }
                                p {class:"pc-muted","One-time payment · No automatic renewal"}
                            }
                        }
                        if p.checkout_snapshot.pricing.promotion_active {
                            div {class:"pc-discount",span{"Original price"}del{"{p.checkout_snapshot.pricing.original_price} {p.token}"}}
                            div {class:"pc-discount",span{"You save"}strong{"{p.checkout_snapshot.pricing.savings} {p.token}"}}
                        }
                        div {class:"pc-total-row",span{"Total due"}strong{"{total}"}}
                        p {class:"pc-fee","Network fees are paid separately in your wallet."}
                        div {class:"pc-trust",crate::primitives::Icon{name:"shield-check",size:18}span{"Your wallet stays in your control."}}
                    }
                }
                section { class:"pc-card",aria_label:"Crypto checkout",
                    if let Some(p)=payment.clone() {
                        div {class:"pc-card-top",span{class:"pc-eyebrow","{p.token} PAYMENT"}span{class:"pc-network","{network(p.chain_id)}"}}
                        if p.environment == Environment::Test {p{class:"pc-test","Test payment · Simulated funds"}}
                    }
                    if phase == Stage::Loading {
                        div {class:"pc-state",role:"status",aria_live:"polite",span{class:"pc-spinner",aria_hidden:true}h2{"Preparing your checkout"}p{"Loading your payment details securely…"}}
                    } else if phase == Stage::Ready {
                        if let Some(p)=payment.clone() {
                            div {class:"pc-pay-intro",h2{"Ready when you are"}p{"Review the amount, then confirm in your wallet."}}
                            div {class:"pc-pay-total","{total}"}
                            p {class:"pc-muted", "{p.description}"}
                            if p.payment_method == "transfer" {
                                div {class:"pc-methods",aria_label:"Payment method",
                                    button{aria_pressed:!wallet_method(),onclick:move |_|wallet_method.set(false),"QR / Transfer"}
                                    button{aria_pressed:wallet_method(),onclick:move |_|wallet_method.set(true),"Wallet"}
                                }
                            }
                            if p.payment_method == "transfer" && !wallet_method() {
                                div {class:"pc-qr",img{src:super::svg_uri(&p.qr_svg),alt:"Payment QR with token, network, amount and recipient"}}
                                CopyField{label:"Payment address · unique to this checkout",text:p.deposit_address.clone()}
                                p {class:"pc-fee","Send exactly {total} on {network(p.chain_id)}."}
                            } else {
                                div {class:"pc-methods",aria_label:"Choose wallet",
                                    button{aria_pressed:!walletconnect(),onclick:move |_|walletconnect.set(false),"MetaMask"}
                                    button{aria_pressed:walletconnect(),onclick:move |_|walletconnect.set(true),"WalletConnect"}
                                }
                                button {class:"pc-primary",disabled:(c.busy)()||!(c.session_loaded)(),onclick:{let payment=p.clone();move |_|pay(c,payment.clone(),String::new(),walletconnect())},"Pay {total}",crate::primitives::Icon{name:"arrow-right",size:20}}
                            }
                            p {class:"pc-fee text-center","You'll review this payment in your wallet before sending."}
                            Expiry{expires:p.expires_at.clone()}
                        }
                    } else {
                        div {class:"pc-state",role:"status",aria_live:"polite",aria_atomic:true,
                            if phase == Stage::Success {span{class:"pc-success-icon",aria_hidden:true,crate::primitives::Icon{name:"check",size:34}}}
                            else if matches!(phase,Stage::Expired|Stage::Refunded|Stage::Review|Stage::Escrow) {span{class:"pc-review-icon",aria_hidden:true,crate::primitives::Icon{name:"info",size:30}}}
                            else {span{class:"pc-spinner",aria_hidden:true}}
                            h2 { {match phase {
                                Stage::Wallet=>match (c.wallet_phase)(){WalletPhase::Connecting=>"Connect your wallet",WalletPhase::Preparing=>"Preparing your payment",WalletPhase::ApproveToken=>"Approve token in your wallet",WalletPhase::ConfirmingApproval=>"Confirming token approval",_=>"Confirm in your wallet"},
                                Stage::Confirming=>"Confirming payment",Stage::Activating=>"Payment confirmed",Stage::Success=>if epsx{"Your plan is ready"}else{"Payment received"},
                                Stage::Escrow=>"Funds held in escrow",Stage::Expired=>"Checkout expired",Stage::Refunded=>"Payment refunded",_=>"Checking your payment"
                            }} }
                            p { {match phase {Stage::Wallet=>"Follow the request in your wallet. This page will update automatically.",Stage::Confirming=>"Your transaction was submitted. We're checking the payment on the network.",Stage::Activating=>if epsx{"Activating your plan. We'll continue as soon as your access is ready."}else{"Checking purchase details. Your payment has been confirmed."},Stage::Success=>if epsx{"Payment complete. Your package is ready to use."}else{"Your payment is confirmed on the network."},Stage::Escrow=>"Your deposit is secured. Manage the escrow below after reviewing your purchase.",Stage::Expired=>"Start a new checkout to make a payment.",Stage::Refunded=>"This payment has been refunded. View your purchase for details.",_=>"Verification is taking longer than usual. Your transaction is preserved."}} }
                            if phase==Stage::Wallet && !pairing().is_empty() {img{class:"pc-pairing",src:pairing_qr(&pairing()),alt:"WalletConnect pairing QR — not a payment QR"}}
                            if phase==Stage::Success {if let Some(p)=payment.clone(){p{class:"font-semibold", "{p.description}"}}}
                            if let Some(n)=countdown() {p{class:"pc-countdown","Returning to your purchase in {n}…"}}
                            if waiting_seconds()>45 {p{class:"pc-long-wait","Still checking. You can keep this page open or return to this checkout later."}}
                        }
                        if phase==Stage::Escrow {if let Some(p)=payment.clone(){OperationButtons{payment:p}}}
                        if let Some(done)=completion.clone() {
                            if phase==Stage::Success || matches!(phase,Stage::Activating|Stage::Refunded|Stage::Review|Stage::Expired) {
                                button{class:if phase==Stage::Success{"pc-primary"}else{"pc-secondary"},onclick:move |_|{spawn(return_now(c));},if phase==Stage::Success{"View purchase now"}else{"View purchase details"}}
                            }
                            span{class:"sr-only","Purchase {done.order_id}"}
                        } else if phase==Stage::Success {
                            if let Some(p)=payment.clone() {Link{class:"pc-primary",to:format!("/m/{}?environment={}",p.merchant_id,p.environment.as_str()),"Back to merchant"}}
                        } else if phase==Stage::Expired {crate::navigation::AppLink{class:"pc-primary",href:format!("{origin}/plans"),"Explore plans"}}
                    }
                    if let Some((chain_id,hash))=transaction {
                        TransactionReceipt{chain_id,hash}
                    }
                    if !message.is_empty() && !matches!(phase,Stage::Success|Stage::Expired|Stage::Refunded) {
                        p {class:"pc-notice",role:"status",aria_live:"polite",
                            if phase==Stage::Confirming {"Your transaction is saved. Checking its latest status…"}
                            else if phase==Stage::Activating {"Payment confirmed. Checking plan activation…"}
                            else {"{message}"}
                        }
                        if !matches!(phase,Stage::Success|Stage::Wallet) {button{class:"pc-text-button",onclick:move |_|c.revision+=1,"Check again"}}
                    }
                    ol {class:"pc-steps",aria_label:"Payment progress",
                        for (i,label) in [(0,"Wallet"),(1,"Payment"),(2,if epsx{"Plan access"}else{"Complete"})] {
                            li {"data-complete":(step>i).to_string(),"data-active":(step==i).to_string(),
                                span{aria_hidden:true,if step>i{"✓"}else{"{i+1}"}}"{label}"
                            }
                        }
                    }
                }
            }
            footer {class:"pc-footer",span{"Payments by EPSX"}div{class:"flex gap-5",crate::navigation::AppLink{href:format!("{origin}/contact"),"Support"}crate::navigation::AppLink{href:format!("{origin}/terms"),"Terms"}crate::navigation::AppLink{href:format!("{origin}/privacy"),"Privacy"}}}
        }
    }
}

#[component]
fn TransactionReceipt(chain_id: u64, hash: String) -> Element {
    let mut copied = use_signal(|| false);
    let url = transaction_explorer_url(chain_id, &hash);
    let short = if valid_hash(&hash) {
        format!("{}…{}", &hash[..10], &hash[hash.len() - 8..])
    } else {
        hash.clone()
    };
    rsx! {div{class:"pc-transaction",
        span{class:"pc-muted","Transaction"}
        div{class:"flex items-center justify-between gap-3",
            if let Some(url)=url {a{href:url,target:"_blank",rel:"noopener noreferrer",aria_label:"View transaction on block explorer (opens in a new tab)",title:hash.clone(),"{short}",crate::primitives::Icon{name:"external-link",size:14}}}
            else {code{class:"break-all","{short}"}}
            button{class:"pc-copy",aria_label:"Copy transaction ID",onclick:move |_|{let hash=hash.clone();spawn(async move{copied.set(wallet::copy(&hash).await.is_ok());});},if copied(){"Copied"}else{"Copy"}}
        }
        if transaction_explorer_url(chain_id,&hash).is_none(){small{class:"pc-muted","Block explorer unavailable for this network"}}
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn paid() -> PageData {
        let order_id = uuid::Uuid::nil();
        PageData {
            frontend_origin: "https://epsx.io".into(),
            completion_available: true,
            payment: Some(Payment {
                status: "succeeded".into(),
                checkout_snapshot: Snapshot {
                    kind: "merchant".into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            completion: Some(CheckoutCompletion {
                order_id,
                payment_status: "succeeded".into(),
                fulfillment_status: "pending".into(),
                return_url: format!("https://epsx.io/account/payments/{order_id}"),
            }),
            ..Default::default()
        }
    }
    #[test]
    fn payment_and_plan_activation_are_separate_visual_states() {
        let mut data = paid();
        let session = CheckoutSession {
            started: true,
            ..Default::default()
        };
        assert_eq!(
            stage(Some(&data), &session, WalletPhase::Submitted),
            Stage::Activating
        );
        assert!(!redirect_ready(Some(&data), &session));
        data.completion.as_mut().unwrap().fulfillment_status = "granted".into();
        assert_eq!(
            stage(Some(&data), &session, WalletPhase::Submitted),
            Stage::Success
        );
        assert!(redirect_ready(Some(&data), &session));
        data.completion_available = false;
        assert!(!redirect_ready(Some(&data), &session));
        data.completion_available = true;
        data.payment.as_mut().unwrap().status = "verification_required".into();
        assert_eq!(
            stage(Some(&data), &session, WalletPhase::Idle),
            Stage::Review
        );
        assert!(!redirect_ready(Some(&data), &session));
    }
    #[test]
    fn old_receipts_and_other_merchants_never_auto_redirect() {
        let mut data = paid();
        data.completion.as_mut().unwrap().fulfillment_status = "granted".into();
        assert!(!redirect_ready(Some(&data), &CheckoutSession::default()));
        assert!(!redirect_ready(
            Some(&data),
            &CheckoutSession {
                started: true,
                redirected: true,
                ..Default::default()
            }
        ));
        data.completion = None;
        assert!(!redirect_ready(
            Some(&data),
            &CheckoutSession {
                started: true,
                ..Default::default()
            }
        ));
        assert_eq!(
            stage(Some(&data), &CheckoutSession::default(), WalletPhase::Idle),
            Stage::Success
        );
    }
    #[test]
    fn submitted_payment_stays_pending_during_refresh_and_outage() {
        let pending = PendingCheckout {
            checkout_id: "cs_fixture".into(),
            chain_id: 56,
            hash: format!("0x{}", "a".repeat(64)),
            operation_id: Some("mop_fixture".into()),
            request_context: "fixture".into(),
        };
        let session = CheckoutSession {
            started: true,
            pending: Some(pending),
            ..Default::default()
        };
        let restored: CheckoutSession =
            serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        assert_eq!(stage(None, &restored, WalletPhase::Idle), Stage::Confirming);
        let mut data = paid();
        data.payment.as_mut().unwrap().status = "awaiting_payment".into();
        assert_eq!(
            stage(Some(&data), &restored, WalletPhase::Idle),
            Stage::Confirming
        );
        assert_eq!(
            stage(
                Some(&data),
                &CheckoutSession::default(),
                WalletPhase::ApproveToken
            ),
            Stage::Wallet
        );
        assert_eq!(
            stage(Some(&data), &CheckoutSession::default(), WalletPhase::Idle),
            Stage::Ready
        );
    }
    #[test]
    fn return_destination_is_fixed_to_the_verified_purchase() {
        let data = paid();
        let mut completion = data.completion.unwrap();
        assert!(completion.valid_return("https://epsx.io"));
        for url in ["https://evil.example/account/payments/00000000-0000-0000-0000-000000000000","https://epsx.io@evil.example/account/payments/00000000-0000-0000-0000-000000000000","https://epsx.io/account/payments/00000000-0000-0000-0000-000000000000?next=https://evil.example","https://epsx.io/auth","javascript:alert(1)"] {
            completion.return_url=url.into();assert!(!completion.valid_return("https://epsx.io"),"{url}");
        }
    }
}
