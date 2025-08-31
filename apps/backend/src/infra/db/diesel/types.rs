use diesel::{deserialize, serialize, sql_types, AsExpression, FromSqlRow};
use diesel::pg::PgValue;
use std::io::Write;
use std::net::IpAddr;
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
    Medium,
    High,
    Critical,
}

impl serialize::ToSql<crate::infra::db::diesel::schema::sql_types::NotificationPriority, diesel::pg::Pg> for NotificationPriority {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        let value = match *self {
            NotificationPriority::Low => "low",
            NotificationPriority::Medium => "medium",
            NotificationPriority::High => "high",
            NotificationPriority::Critical => "critical",
        };
        out.write_all(value.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::infra::db::diesel::schema::sql_types::NotificationPriority, diesel::pg::Pg> for NotificationPriority {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match std::str::from_utf8(bytes.as_bytes())? {
            "low" => Ok(NotificationPriority::Low),
            "medium" => Ok(NotificationPriority::Medium),
            "high" => Ok(NotificationPriority::High),
            "critical" => Ok(NotificationPriority::Critical),
            _ => Err("Unrecognized NotificationPriority variant".into()),
        }
    }
}

impl std::fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationPriority::Low => write!(f, "low"),
            NotificationPriority::Medium => write!(f, "medium"),
            NotificationPriority::High => write!(f, "high"),
            NotificationPriority::Critical => write!(f, "critical"),
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
    Info,
    Warning,
    Error,
    Success,
    FeatureAccess,
    RoleChange,
    SystemUpdate,
}

impl serialize::ToSql<crate::infra::db::diesel::schema::sql_types::NotificationType, diesel::pg::Pg> for NotificationType {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        let value = match *self {
            NotificationType::Info => "info",
            NotificationType::Warning => "warning",
            NotificationType::Error => "error",
            NotificationType::Success => "success",
            NotificationType::FeatureAccess => "feature_access",
            NotificationType::RoleChange => "role_change",
            NotificationType::SystemUpdate => "system_update",
        };
        out.write_all(value.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::infra::db::diesel::schema::sql_types::NotificationType, diesel::pg::Pg> for NotificationType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match std::str::from_utf8(bytes.as_bytes())? {
            "info" => Ok(NotificationType::Info),
            "warning" => Ok(NotificationType::Warning),
            "error" => Ok(NotificationType::Error),
            "success" => Ok(NotificationType::Success),
            "feature_access" => Ok(NotificationType::FeatureAccess),
            "role_change" => Ok(NotificationType::RoleChange),
            "system_update" => Ok(NotificationType::SystemUpdate),
            _ => Err("Unrecognized NotificationType variant".into()),
        }
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Info => write!(f, "info"),
            NotificationType::Warning => write!(f, "warning"),
            NotificationType::Error => write!(f, "error"),
            NotificationType::Success => write!(f, "success"),
            NotificationType::FeatureAccess => write!(f, "feature_access"),
            NotificationType::RoleChange => write!(f, "role_change"),
            NotificationType::SystemUpdate => write!(f, "system_update"),
        }
    }
}