use diesel::{deserialize, serialize, sql_types, AsExpression, FromSqlRow};
use std::net::IpAddr;
use diesel::pg::PgValue;
use std::io::Write;
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

// Custom IpAddr type for Diesel PostgreSQL Inet support
#[derive(Debug, Clone, AsExpression, FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = sql_types::Inet)]
#[serde(transparent)]
pub struct DieselIpAddr(pub IpAddr);

impl From<IpAddr> for DieselIpAddr {
    fn from(ip: IpAddr) -> Self {
        DieselIpAddr(ip)
    }
}

impl From<DieselIpAddr> for IpAddr {
    fn from(diesel_ip: DieselIpAddr) -> Self {
        diesel_ip.0
    }
}

impl std::fmt::Display for DieselIpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for DieselIpAddr {
    type Err = std::net::AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ip_addr: IpAddr = s.parse()?;
        Ok(DieselIpAddr(ip_addr))
    }
}

impl serialize::ToSql<sql_types::Inet, diesel::pg::Pg> for DieselIpAddr {
    fn to_sql<'b>(
        &'b self,
        out: &mut serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> serialize::Result {
        let ip_str = self.0.to_string();
        out.write_all(ip_str.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<sql_types::Inet, diesel::pg::Pg> for DieselIpAddr {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let ip_str = std::str::from_utf8(bytes.as_bytes())?;
        let ip_addr: IpAddr = ip_str.parse()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        Ok(DieselIpAddr(ip_addr))
    }
}

// Custom Decimal type for Diesel PostgreSQL Numeric support  
#[derive(Debug, Clone, AsExpression, FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = sql_types::Numeric)]
#[serde(transparent)]
pub struct DieselDecimal(pub Decimal);

impl From<Decimal> for DieselDecimal {
    fn from(decimal: Decimal) -> Self {
        DieselDecimal(decimal)
    }
}

impl From<DieselDecimal> for Decimal {
    fn from(diesel_decimal: DieselDecimal) -> Self {
        diesel_decimal.0
    }
}

impl serialize::ToSql<sql_types::Numeric, diesel::pg::Pg> for DieselDecimal {
    fn to_sql<'b>(
        &'b self,
        out: &mut serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> serialize::Result {
        let decimal_str = self.0.to_string();
        out.write_all(decimal_str.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<sql_types::Numeric, diesel::pg::Pg> for DieselDecimal {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let decimal_str = std::str::from_utf8(bytes.as_bytes())?;
        let decimal: Decimal = decimal_str.parse()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        Ok(DieselDecimal(decimal))
    }
}

// ============================================================================
// PACKAGE TIER - Now using VARCHAR instead of enum
// ============================================================================
// The package_tier column is now VARCHAR(50) to allow flexible tier values
// like 'FREE', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'admin', etc.
// No custom Diesel type needed since it's a simple string.


// ============================================================================
// NOTIFICATION PRIORITY ENUM
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::infra::db::diesel::schema::sql_types::NotificationPriority)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl serialize::ToSql<crate::infra::db::diesel::schema::sql_types::NotificationPriority, diesel::pg::Pg> for NotificationPriority {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        let value = match *self {
            NotificationPriority::Low => "low",
            NotificationPriority::Normal => "normal",
            NotificationPriority::High => "high",
            NotificationPriority::Urgent => "urgent",
        };
        out.write_all(value.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::infra::db::diesel::schema::sql_types::NotificationPriority, diesel::pg::Pg> for NotificationPriority {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match std::str::from_utf8(bytes.as_bytes())? {
            "low" => Ok(NotificationPriority::Low),
            "normal" => Ok(NotificationPriority::Normal),
            "high" => Ok(NotificationPriority::High),
            "urgent" => Ok(NotificationPriority::Urgent),
            _ => Err("Unrecognized NotificationPriority variant".into()),
        }
    }
}

impl std::fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationPriority::Low => write!(f, "low"),
            NotificationPriority::Normal => write!(f, "normal"),
            NotificationPriority::High => write!(f, "high"),
            NotificationPriority::Urgent => write!(f, "urgent"),
        }
    }
}

// ============================================================================
// NOTIFICATION TYPE ENUM
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::infra::db::diesel::schema::sql_types::NotificationType)]
pub enum NotificationType {
    System,
    Admin,
    Security,
    Feature,
    Marketing,
}

impl serialize::ToSql<crate::infra::db::diesel::schema::sql_types::NotificationType, diesel::pg::Pg> for NotificationType {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        let value = match *self {
            NotificationType::System => "system",
            NotificationType::Admin => "admin",
            NotificationType::Security => "security",
            NotificationType::Feature => "feature",
            NotificationType::Marketing => "marketing",
        };
        out.write_all(value.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::infra::db::diesel::schema::sql_types::NotificationType, diesel::pg::Pg> for NotificationType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match std::str::from_utf8(bytes.as_bytes())? {
            "system" => Ok(NotificationType::System),
            "admin" => Ok(NotificationType::Admin),
            "security" => Ok(NotificationType::Security),
            "feature" => Ok(NotificationType::Feature),
            "marketing" => Ok(NotificationType::Marketing),
            _ => Err("Unrecognized NotificationType variant".into()),
        }
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::System => write!(f, "system"),
            NotificationType::Admin => write!(f, "admin"),
            NotificationType::Security => write!(f, "security"),
            NotificationType::Feature => write!(f, "feature"),
            NotificationType::Marketing => write!(f, "marketing"),
        }
    }
}

// ============================================================================
// DELIVERY CHANNEL ENUM
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::infra::db::diesel::schema::sql_types::DeliveryChannel)]
pub enum DeliveryChannel {
    FcmPush,
    InApp,
    Email,
    Sms,
}

impl serialize::ToSql<crate::infra::db::diesel::schema::sql_types::DeliveryChannel, diesel::pg::Pg> for DeliveryChannel {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        let value = match *self {
            DeliveryChannel::FcmPush => "fcm_push",
            DeliveryChannel::InApp => "in_app",
            DeliveryChannel::Email => "email",
            DeliveryChannel::Sms => "sms",
        };
        out.write_all(value.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::infra::db::diesel::schema::sql_types::DeliveryChannel, diesel::pg::Pg> for DeliveryChannel {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match std::str::from_utf8(bytes.as_bytes())? {
            "fcm_push" => Ok(DeliveryChannel::FcmPush),
            "in_app" => Ok(DeliveryChannel::InApp),
            "email" => Ok(DeliveryChannel::Email),
            "sms" => Ok(DeliveryChannel::Sms),
            _ => Err("Unrecognized DeliveryChannel variant".into()),
        }
    }
}

impl std::fmt::Display for DeliveryChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryChannel::FcmPush => write!(f, "fcm_push"),
            DeliveryChannel::InApp => write!(f, "in_app"),
            DeliveryChannel::Email => write!(f, "email"),
            DeliveryChannel::Sms => write!(f, "sms"),
        }
    }
}

// ============================================================================
// DELIVERY STATUS ENUM
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::infra::db::diesel::schema::sql_types::DeliveryStatus)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Expired,
}

impl serialize::ToSql<crate::infra::db::diesel::schema::sql_types::DeliveryStatus, diesel::pg::Pg> for DeliveryStatus {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        let value = match *self {
            DeliveryStatus::Pending => "pending",
            DeliveryStatus::Sent => "sent",
            DeliveryStatus::Delivered => "delivered",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::Expired => "expired",
        };
        out.write_all(value.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::infra::db::diesel::schema::sql_types::DeliveryStatus, diesel::pg::Pg> for DeliveryStatus {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match std::str::from_utf8(bytes.as_bytes())? {
            "pending" => Ok(DeliveryStatus::Pending),
            "sent" => Ok(DeliveryStatus::Sent),
            "delivered" => Ok(DeliveryStatus::Delivered),
            "failed" => Ok(DeliveryStatus::Failed),
            "expired" => Ok(DeliveryStatus::Expired),
            _ => Err("Unrecognized DeliveryStatus variant".into()),
        }
    }
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryStatus::Pending => write!(f, "pending"),
            DeliveryStatus::Sent => write!(f, "sent"),
            DeliveryStatus::Delivered => write!(f, "delivered"),
            DeliveryStatus::Failed => write!(f, "failed"),
            DeliveryStatus::Expired => write!(f, "expired"),
        }
    }
}