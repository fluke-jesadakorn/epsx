use super::*;
use crate::fullstack::frontend_auth::browser;
#[derive(Clone, Copy)]
struct Controller {
    busy: Signal<bool>,
    message: Signal<String>,
    revision: Signal<u64>,
}
impl Controller {
    fn submit(mut self, command: WalletCommand) {
        if (self.busy)() {
            return;
        }
        self.busy.set(true);
        self.message.set("Saving…".into());
        spawn(async move {
            let result: Result<(), String> = async {
                let context = serde_json::to_string(&command).map_err(|e| e.to_string())?;
                let key: String = browser("key", &context).await?;
                let response = wallets_action(command, key)
                    .await
                    .map_err(|e| e.to_string())?
                    .map_err(|e| e.message().to_string())?;
                self.message.set(response.message);
                if response.committed {
                    let _: bool = browser("complete", &context).await?;
                    self.revision += 1;
                }
                Ok(())
            }
            .await;
            if let Err(message) = result {
                self.message.set(message);
            }
            self.busy.set(false);
        });
    }
}
#[component]
pub fn HydratedAdminWallets(path: String, query: String) -> Element {
    let valid_filter = crate::pages::admin_pages::wallet_wallets::AdminWalletListQuery::from_raw(
        query.trim_start_matches('?'),
    )
    .is_ok();
    let filter = crate::pages::admin_pages::wallet_wallets::AdminWalletListQuery::from_raw(
        query.trim_start_matches('?'),
    )
    .ok()
    .map(|q| WalletFilter {
        search: q.search,
        status: q.status,
        page: q.page,
        limit: q.limit,
    })
    .unwrap_or_default();
    let request = WalletRequest {
        page: if WalletPage::from_path(&path) == WalletPage::List && !valid_filter {
            WalletPage::InvalidQuery
        } else {
            WalletPage::from_path(&path)
        },
        filter,
    };
    let first = request.clone();
    let initial = use_server_future(move || wallets_read(first.clone()))?;
    let mut data = use_signal(|| {
        initial
            .read()
            .as_ref()
            .cloned()
            .and_then(Result::ok)
            .unwrap_or(Err(LoadError::Unavailable))
    });
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let busy = use_signal(|| false);
    let message = use_signal(String::new);
    let revision = use_signal(|| 0u64);
    let controller = Controller {
        busy,
        message,
        revision,
    };
    use_context_provider(|| controller);
    let refresh = request.clone();
    use_effect(move || {
        let current = revision();
        if current == 0 {
            return;
        }
        let refresh = refresh.clone();
        loading.set(true);
        spawn(async move {
            let result = wallets_read(refresh).await;
            if current != *revision.peek() {
                return;
            }
            match result {
                Ok(Ok(value)) => {
                    data.set(Ok(value));
                    error.set(None);
                }
                Ok(Err(value)) => error.set(Some(value.message().into())),
                Err(_) => error.set(Some("Could not refresh wallet data.".into())),
            }
            loading.set(false);
        });
    });
    if let Err(value) = data() {
        crate::pages::news::hydrated::response_status(match value {
            LoadError::Unauthenticated => 401,
            LoadError::Forbidden => 403,
            LoadError::NotFound => 404,
            LoadError::InvalidQuery => 400,
            _ => 502,
        });
    }
    rsx! {crate::fullstack::admin::AdminAnalyticsShell{authenticated:data().is_ok(),current_path:path,title:"Wallet Management",
    main{class:"container-x max-w-7xl mx-auto p-6 space-y-6","data-dioxus-admin-wallets":"true",
     header{class:"space-y-2",h1{class:"text-3xl font-bold","Wallet Management Hub"}p{class:"text-muted-foreground","Manage wallets, permissions, credits, and subscription plans."}}
     nav{class:"flex flex-wrap gap-3",for(label,to)in[("Wallets","/wallet-management/wallets"),("Access","/wallet-management/access"),("Credits","/wallet-management/credits"),("Plans","/wallet-management/access/plans")]{Link{class:"btn btn-outline",to,"{label}"}}button{class:"btn btn-outline",disabled:loading()||busy(),onclick:move |_|{let mut value=revision;value+=1;},"Refresh"}}
     if !message().is_empty(){p{role:"status","aria-live":"polite","{message}"}}if let Some(value)=error(){p{role:"alert",crate::fullstack::load_error::SessionMessage{message:value}}}
     match data(){Err(value)=>rsx!{section{class:"rounded-2xl border border-border/30 p-6",role:"alert",crate::fullstack::load_error::LoadErrorNotice { error: value.clone(),  }}},Ok(value)=>rsx!{
      if let Ok(stats)=value.stats{section{class:"grid gap-4 sm:grid-cols-3",for(label,count)in[("Total wallets",stats.total_users),("Active users",stats.active_users),("Disabled",stats.inactive_users)]{article{class:"rounded-2xl border border-border/30 bg-card p-5",p{class:"text-2xl font-bold","{count}"}p{class:"text-muted-foreground","{label}"}}}}}
      match request.page.clone(){
       WalletPage::List=>rsx!{WalletList{projection:value.wallets,filter:request.filter.clone()}},
       WalletPage::Detail(_)=>rsx!{if let Some(detail)=value.detail{section{class:"rounded-2xl border border-border/30 bg-card p-6 space-y-3",h2{class:"text-xl font-bold","Wallet details"}p{class:"break-all","{detail.address}"}p{"Chain {detail.chain_id} · {detail.status} · Version {detail.version}"}p{"{detail.label.clone().unwrap_or_default()}"}if detail.status=="active"{Link{class:"btn btn-outline",to:WalletPage::Disable(detail.address.clone()).path(),"Disable wallet"}}}AccessSection{projection:value.access,wallet:Some(detail.address),plans:value.plans}}},
       WalletPage::Disable(_)=>rsx!{if let Some(detail)=value.detail{section{class:"rounded-2xl border border-border/30 bg-card p-6 space-y-4",h2{class:"text-xl font-bold","Disable wallet"}p{class:"break-all","{detail.address}"}p{"Disabling this wallet blocks access until it is re-enabled by the backend."}if detail.status=="active"{MutationForm{kind:FormKind::Disable(detail)}}else{p{"This wallet is already disabled."}}}}},
       WalletPage::Access=>rsx!{AccessSection{projection:value.access,wallet:None,plans:None}},
       WalletPage::Credits=>rsx!{if let Some(credits)=value.credits{section{class:"grid gap-4 sm:grid-cols-2 lg:grid-cols-4",for(label,count)in[("Outstanding credits",credits.outstanding_minor),("Granted today",credits.granted_today_minor),("Revoked today",credits.revoked_today_minor),("Active accounts",credits.active_accounts)]{article{class:"rounded-xl border border-border/30 p-5",h2{"{label}"}p{class:"text-xl font-bold","{count}"}}}}}MutationForm{kind:FormKind::Credit(false)}MutationForm{kind:FormKind::Credit(true)}},
       WalletPage::Plans=>rsx!{section{class:"rounded-2xl border border-border/30 bg-card p-6 space-y-3",h2{class:"text-xl font-bold","Subscription plans"}if let Some(plans)=value.plans{if plans.items.is_empty(){p{"No plans yet."}}for plan in plans.items{Link{class:"block border-b border-border/30 py-3",to:WalletPage::Plan(plan.id).path(),"{plan.name} · {plan.amount} {plan.currency}"}}}MutationForm{kind:FormKind::Plan(None)}}},
       WalletPage::Plan(_)=>rsx!{if let Some(plan)=value.plan{MutationForm{kind:FormKind::Plan(Some(plan))}}},
       WalletPage::InvalidQuery=>rsx!{p{"Check the wallet filters."}},
    WalletPage::NotFound=>rsx!{p{"Wallet page not found."}},
      }
     }}
    }
    }}
}
#[component]
fn WalletList(projection: Option<AdminWalletListProjection>, filter: WalletFilter) -> Element {
    let navigator = use_navigator();
    let current = filter.clone();
    rsx! {section{class:"rounded-2xl border border-border/30 bg-card p-6 space-y-4",
    h2{class:"text-xl font-bold","Wallet inventory"}
    form{class:"flex flex-wrap gap-3",onsubmit:move|event|{event.prevent_default();let fields=Values::new(&event);let mut next=current.clone();next.search=fields.optional("search");next.status=fields.optional("status");next.page=1;navigator.push(next.url());},label{"Search wallet" input{class:"input",name:"search",value:filter.search.clone().unwrap_or_default(),maxlength:42}}label{"Status" select{class:"input",name:"status",value:filter.status.clone().unwrap_or_default(),option{value:"",selected:filter.status.is_none(),"All"}option{value:"active",selected:filter.status.as_deref()==Some("active"),"Active"}option{value:"disabled",selected:filter.status.as_deref()==Some("disabled"),"Disabled"}}}button{class:"btn btn-primary",r#type:"submit","Apply filters"}}
    if let Some(projection)=projection{div{class:"overflow-x-auto",table{class:"w-full text-left",thead{tr{for label in["Wallet","Chain","Status","Role","Version"]{th{class:"p-3","{label}"}}}}tbody{for item in projection.items{tr{class:"border-t border-border/30",td{class:"p-3 break-all",Link{to:WalletPage::Detail(item.address.clone()).path(),"{item.label.clone().unwrap_or_else(|| item.address.clone())}"}}td{class:"p-3","{item.chain_id}"}td{class:"p-3","{item.status}"}td{class:"p-3","{item.role.clone().unwrap_or_default()}"}td{class:"p-3","{item.version}"}}}}}}
    if projection.total==0{p{"No wallets match these filters."}}
    div{class:"flex flex-wrap items-center gap-3",p{"{projection.total} wallets · Page {filter.page}"}select{class:"input",aria_label:"Wallets per page",value:filter.limit.to_string(),onchange:{let filter=filter.clone();move|event|{let mut next=filter.clone();next.limit=event.value().parse().unwrap_or(10);next.page=1;navigator.push(next.url());}},for value in[10,25,50]{option{value:value.to_string(),selected:filter.limit==value,"{value}"}}}button{class:"btn btn-outline",disabled:filter.page<=1,onclick:{let filter=filter.clone();move |_|{let mut next=filter.clone();next.page-=1;navigator.push(next.url());}},"Previous"}button{class:"btn btn-outline",disabled:projection.offset+projection.limit>=projection.total,onclick:{let filter=filter.clone();move |_|{let mut next=filter.clone();next.page+=1;navigator.push(next.url());}},"Next"}}
    }
    }}
}
#[component]
fn AccessSection(
    projection: Option<AdminAccessProjection>,
    wallet: Option<String>,
    plans: Option<AdminPlanListProjection>,
) -> Element {
    let controller = use_context::<Controller>();
    rsx! {section{class:"rounded-2xl border border-border/30 bg-card p-6 space-y-4",h2{class:"text-xl font-bold","Access assignments"}
     if let Some(projection)=projection{if projection.items.is_empty(){p{"No access assignments."}}for item in projection.items{article{class:"border-b border-border/30 py-3 space-y-2",p{class:"break-all","{item.wallet_address}"}p{"{item.plan_name} · {item.permission} · Version {item.version}"}if let Some(expires)=item.expires_at{p{"Expires {expires}"}}button{class:"btn btn-outline",disabled:(controller.busy)(),onclick:move |_|controller.submit(WalletCommand::Access{revoke:true,wallet_address:item.wallet_address.clone(),plan_id:item.plan_id.clone(),permission:item.permission.clone(),expected_version:item.version}),"Revoke assignment"}}}}else{p{role:"alert","Access assignments are unavailable. Refresh to try again."}}
     if let Some(plans)=plans{details{summary{"Available plans"}for plan in plans.items{p{class:"break-all","{plan.name}: {plan.id}"}}}}
     MutationForm{kind:FormKind::Access(wallet)}
    }}
}
#[derive(Clone, PartialEq)]
enum FormKind {
    Access(Option<String>),
    Credit(bool),
    Plan(Option<AdminPlanProjection>),
    Disable(AdminWalletDetailProjection),
}
struct Values(std::collections::BTreeMap<String, String>);
impl Values {
    fn new(event: &FormEvent) -> Self {
        Self(
            event
                .values()
                .into_iter()
                .filter_map(|(key, value)| match value {
                    dioxus::html::FormValue::Text(value) => Some((key, value)),
                    _ => None,
                })
                .collect(),
        )
    }
    fn get(&self, key: &str) -> String {
        self.0.get(key).cloned().unwrap_or_default()
    }
    fn optional(&self, key: &str) -> Option<String> {
        let value = self.get(key);
        (!value.is_empty()).then_some(value)
    }
    fn number(&self, key: &str) -> Result<i64, String> {
        self.get(key)
            .parse()
            .map_err(|_| format!("Enter a valid {key}."))
    }
}
#[component]
fn Field(
    label: String,
    name: String,
    #[props(default)] value: String,
    #[props(default)] optional: bool,
) -> Element {
    let mut text = use_signal(|| value);
    rsx! {label{class:"block space-y-1",span{"{label}"}input{class:"input w-full",name,value:text(),oninput:move |event| text.set(event.value()),required:!optional}}}
}
#[component]
fn MutationForm(kind: FormKind) -> Element {
    let mut controller = use_context::<Controller>();
    let submit = kind.clone();
    let mut active = use_signal(|| match &kind {
        FormKind::Plan(Some(plan)) => plan.active.unwrap_or(true),
        _ => true,
    });
    let title = match &kind {
        FormKind::Access(_) => "Assign access",
        FormKind::Credit(false) => "Grant credits",
        FormKind::Credit(true) => "Revoke credits",
        FormKind::Plan(None) => "Create subscription plan",
        FormKind::Plan(Some(_)) => "Edit subscription plan",
        FormKind::Disable(_) => "Confirm disable",
    };
    rsx! {form{class:"rounded-xl border border-border/30 p-5 space-y-4",onsubmit:move|event|{event.prevent_default();let values=Values::new(&event);let result:Result<WalletCommand,String>=(||Ok(match submit.clone(){FormKind::Access(wallet)=>WalletCommand::Access{revoke:false,wallet_address:wallet.unwrap_or_else(||values.get("wallet_address")),plan_id:values.get("plan_id"),permission:values.get("permission"),expected_version:values.number("expected_version")?},FormKind::Credit(revoke)=>WalletCommand::Credit{revoke,wallet_address:values.get("wallet_address"),amount_minor:values.number("amount_minor")?,reason:values.get("reason"),expected_version:values.number("expected_version")?},FormKind::Disable(detail)=>WalletCommand::Disable{address:detail.address,reason:values.get("reason"),expected_version:detail.version},FormKind::Plan(plan)=>WalletCommand::Plan{plan_id:plan.as_ref().map(|p|p.id.clone()),merchant_id:values.optional("merchant_id"),name:values.get("name"),description:values.get("description"),amount:values.get("amount"),currency:values.get("currency"),chain_id:values.get("chain_id"),interval:i32::try_from(values.number("interval")?).map_err(|_|"Interval is too large")?,active:Some(values.get("active")!="false"),expected_version:plan.map(|p|p.version)}}))();match result{Ok(command)=>controller.submit(command),Err(error)=>controller.message.set(error)}},
    h3{class:"text-lg font-semibold","{title}"}
    div{class:"grid gap-3 sm:grid-cols-2",match kind{
     FormKind::Access(wallet)=>rsx!{if wallet.is_none(){Field{label:"Wallet address",name:"wallet_address"}}Field{label:"Plan UUID",name:"plan_id"}Field{label:"Permission",name:"permission"}Field{label:"Expected version",name:"expected_version"}},
     FormKind::Credit(_)=>rsx!{Field{label:"Wallet address",name:"wallet_address"}Field{label:"Amount in minor units",name:"amount_minor"}Field{label:"Expected version",name:"expected_version"}Field{label:"Reason",name:"reason"}},
     FormKind::Disable(_)=>rsx!{Field{label:"Reason for disabling",name:"reason"}},
     FormKind::Plan(plan)=>rsx!{if plan.is_none(){Field{label:"Merchant UUID",name:"merchant_id"}}Field{label:"Plan name",name:"name",value:plan.as_ref().map(|p|p.name.clone()).unwrap_or_default()}Field{label:"Description",name:"description",optional:true,value:plan.as_ref().and_then(|p|p.description.clone()).unwrap_or_default()}Field{label:"Amount in minor units",name:"amount",value:plan.as_ref().map(|p|p.amount.clone()).unwrap_or_default()}Field{label:"Currency",name:"currency",value:plan.as_ref().map(|p|p.currency.clone()).unwrap_or_default()}Field{label:"Chain ID",name:"chain_id",value:plan.as_ref().map(|p|p.chain_id.clone()).unwrap_or_default()}Field{label:"Interval in days",name:"interval",value:plan.as_ref().map(|p|p.interval.to_string()).unwrap_or_default()}label{"Active" select{class:"input",name:"active",value:active().to_string(),onchange:move |event| active.set(event.value()=="true"),option{value:"true",selected:active(),"Active"}option{value:"false",selected:!active(),"Inactive"}}}},
    }}button{class:"btn btn-primary",r#type:"submit",disabled:(controller.busy)(),"{title}"}
    }}
}
