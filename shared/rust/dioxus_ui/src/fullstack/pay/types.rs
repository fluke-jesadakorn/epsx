use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Test,
    Live,
}
impl Environment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Live => "live",
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Credentials {
    pub environment: Environment,
    pub checkout: Option<String>,
    pub capability: Option<String>,
    pub guest: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Page {
    Dashboard,
    Payments,
    Packages,
    EditPackage(String),
    Links,
    Webhooks,
    Settings,
    Store(String),
    Product(String),
    Checkout(String),
    Payment(String),
    Link(String),
    NativeIntent(String),
    NativeLink(String),
    NativeDashboard,
}
impl Page {
    pub fn public(&self) -> bool {
        matches!(
            self,
            Self::Store(_)
                | Self::Product(_)
                | Self::Checkout(_)
                | Self::Link(_)
                | Self::NativeLink(_)
        )
    }
    pub fn title(&self) -> &'static str {
        match self {
            Self::Dashboard => "Overview",
            Self::Payments => "Payments",
            Self::Packages | Self::EditPackage(_) => "Packages",
            Self::Links => "Payment links",
            Self::Webhooks => "Webhooks",
            Self::Settings => "Settings",
            Self::Store(_) | Self::Product(_) => "Explore the collection",
            Self::Checkout(_) => "Checkout",
            Self::Payment(_) | Self::NativeIntent(_) => "Payment details",
            Self::Link(_) | Self::NativeLink(_) => "Payment link",
            Self::NativeDashboard => "Payments with escrow",
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Merchant {
    #[serde(alias = "id")]
    pub merchant_id: String,
    pub name: String,
    #[serde(alias = "owner")]
    pub wallet: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub merchant_name: String,
    #[serde(deserialize_with = "null_default")]
    pub description: String,
    pub duration_days: Option<u32>,
    pub enabled: bool,
    pub prices: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pricing {
    pub promotion_active: bool,
    pub original_price: String,
    pub savings: String,
    pub promotion_discount: f64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Snapshot {
    pub kind: String,
    pub merchant_name: String,
    pub item_name: String,
    #[serde(deserialize_with = "null_default")]
    pub description: String,
    pub duration_days: Option<u32>,
    pub pricing: Pricing,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Payment {
    pub id: String,
    pub checkout_id: String,
    pub merchant_id: String,
    pub environment: Environment,
    #[serde(deserialize_with = "null_default")]
    pub description: String,
    pub amount: String,
    #[serde(alias = "token_symbol")]
    pub token: String,
    pub token_decimals: Option<u32>,
    pub payee: String,
    pub mode: String,
    pub settlement_status: String,
    pub status: String,
    pub payment_method: String,
    pub deposit_address: String,
    pub chain_id: u64,
    pub qr_svg: String,
    pub expires_at: String,
    pub tx_hash: Option<String>,
    pub available_actions: Vec<String>,
    pub checkout_snapshot: Snapshot,
}
impl Payment {
    pub fn terminal(&self) -> bool {
        matches!(
            self.status.as_str(),
            "succeeded" | "refunded" | "expired" | "verification_required"
        )
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaymentLink {
    #[serde(alias = "slug")]
    pub id: String,
    #[serde(deserialize_with = "null_default")]
    pub description: String,
    pub amount: String,
    pub token: String,
    pub mode: String,
    pub disabled: bool,
    pub url: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub revoked: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Webhook {
    pub id: String,
    pub url: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Delivery {
    pub id: String,
    pub event_id: String,
    pub status: String,
    pub attempts: u32,
    pub last_status: Option<u16>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Overview {
    pub token: String,
    pub decimals: u32,
    pub paid: String,
    pub ready: String,
    pub fees: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenConfig {
    pub decimals: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub environment: Environment,
    pub tokens: BTreeMap<String, TokenConfig>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub environments: Vec<NetworkConfig>,
    pub tokens: BTreeMap<String, TokenConfig>,
}
impl Config {
    pub fn decimals(&self, env: Environment, token: &str) -> Option<u32> {
        self.environments
            .iter()
            .find(|n| n.environment == env)
            .and_then(|n| n.tokens.get(token))
            .or_else(|| self.tokens.get(token))
            .map(|t| t.decimals)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PageData {
    pub signed_in: bool,
    pub recover_session: bool,
    pub frontend_origin: String,
    pub merchant: Option<Merchant>,
    pub config: Config,
    pub products: Vec<Product>,
    pub payments: Vec<Payment>,
    pub links: Vec<PaymentLink>,
    pub keys: Vec<ApiKey>,
    pub webhooks: Vec<Webhook>,
    pub deliveries: Vec<Delivery>,
    pub overview: Vec<Overview>,
    pub payment: Option<Payment>,
    pub link: Option<PaymentLink>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductInput {
    pub name: String,
    #[serde(deserialize_with = "null_default")]
    pub description: String,
    pub prices: BTreeMap<String, String>,
    pub duration_days: Option<u32>,
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkInput {
    pub mode: String,
    pub token: String,
    pub amount: String,
    #[serde(deserialize_with = "null_default")]
    pub description: String,
    pub max_uses: Option<u32>,
    pub expires_in: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PayAuthCommand {
    Challenge {
        address: String,
    },
    Verify {
        address: String,
        message: String,
        nonce: String,
        signature: String,
    },
    Refresh,
    Logout,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayChallenge {
    pub address: String,
    pub message: String,
    pub nonce: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Auth(PayAuthCommand),
    NativeCreateLink(LinkInput),
    NativeRedeem {
        id: String,
    },
    NativePrepare {
        id: String,
        kind: OperationKind,
    },
    NativeConfirm {
        id: String,
        tx_hash: String,
    },
    NativeRead {
        id: String,
    },
    Register {
        name: String,
    },
    Profile {
        name: String,
    },
    SaveProduct {
        id: Option<String>,
        input: ProductInput,
    },
    CreateLink(LinkInput),
    CreateKey {
        name: String,
    },
    RevokeKey {
        id: String,
    },
    CreateWebhook {
        url: String,
    },
    DisableWebhook {
        id: String,
    },
    EnableWebhook {
        id: String,
    },
    RotateWebhook {
        id: String,
    },
    ReplaceWebhook {
        id: String,
        url: String,
    },
    ReplayDelivery {
        id: String,
    },
    DeliveryDetails {
        id: String,
    },
    DisableLink {
        id: String,
    },
    BuyProduct {
        id: String,
        token: String,
    },
    RedeemLink {
        id: String,
    },
    PrepareTransfer {
        id: String,
        payer: String,
    },
    PrepareOperation {
        id: String,
        checkout: bool,
        kind: OperationKind,
        payer: String,
    },
    ConfirmOperation {
        id: String,
        tx_hash: String,
    },
    ReadOperation {
        id: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationKind {
    Pay,
    Deposit,
    Release,
    Refund,
    Dispute,
    Collect,
}
impl OperationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pay => "pay",
            Self::Deposit => "deposit",
            Self::Release => "release",
            Self::Refund => "refund",
            Self::Dispute => "dispute",
            Self::Collect => "collect",
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: String,
    #[serde(rename = "chainId")]
    pub chain_id: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionResult {
    pub challenge: Option<PayChallenge>,
    pub id: String,
    pub status: String,
    pub key: Option<String>,
    pub signing_secret: Option<String>,
    pub url: Option<String>,
    pub pay_url: Option<String>,
    #[serde(alias = "transaction")]
    pub transaction_parameters: Option<Transaction>,
    pub approval_transaction: Option<Transaction>,
    pub payment: Option<Payment>,
    pub intent: Option<Payment>,
    pub operation: Option<OperationReference>,
    pub attempts: Vec<DeliveryAttempt>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DeliveryAttempt {
    pub status: Option<u16>,
    pub response_body: Option<String>,
    pub attempted_at: Option<String>,
}

/// Token formatting is presentation only; amounts and validation remain in Pay.
pub fn display_amount(value: &str, decimals: u32) -> String {
    if value.is_empty() {
        return "0".into();
    }
    let decimals = decimals.min(36) as usize;
    if decimals == 0 {
        return value.into();
    }
    let padded = format!("{:0>width$}", value, width = decimals + 1);
    let (whole, fraction) = padded.split_at(padded.len() - decimals);
    let fraction = fraction.trim_end_matches('0');
    if fraction.is_empty() {
        whole.into()
    } else {
        format!("{whole}.{fraction}")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OperationReference {
    pub id: String,
}
fn null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
