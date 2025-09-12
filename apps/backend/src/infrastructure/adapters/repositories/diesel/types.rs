// Diesel custom types and enums
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Notification type enum
#[derive(Debug, Clone, Serialize, Deserialize, diesel::AsExpression, diesel::FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
    Feature,
    Security,
    System,
    Admin,
    General,
    Marketing,
}

/// Notification priority enum
#[derive(Debug, Clone, Serialize, Deserialize, diesel::AsExpression, diesel::FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
    Urgent,
}

/// Delivery channel enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, diesel::AsExpression, diesel::FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum DeliveryChannel {
    Email,
    Push,
    SMS,
    Sms,  // Alternative casing for compatibility
    FcmPush,  // FCM-specific push notifications
    InApp,
}

// Implement ToSql and FromSql traits for the enums
impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg> for NotificationType {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        let value = match self {
            NotificationType::Info => "info",
            NotificationType::Warning => "warning", 
            NotificationType::Error => "error",
            NotificationType::Success => "success",
            NotificationType::Feature => "feature",
            NotificationType::Security => "security",
            NotificationType::System => "system",
            NotificationType::Admin => "admin",
            NotificationType::General => "general",
            NotificationType::Marketing => "marketing",
        };
        <str as diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg>>::to_sql(value, out)
    }
}

impl diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for NotificationType {
    fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match <String as diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg>>::from_sql(bytes)?.as_str() {
            "info" => Ok(NotificationType::Info),
            "warning" => Ok(NotificationType::Warning),
            "error" => Ok(NotificationType::Error),
            "success" => Ok(NotificationType::Success),
            "feature" => Ok(NotificationType::Feature),
            "security" => Ok(NotificationType::Security),
            "system" => Ok(NotificationType::System),
            "admin" => Ok(NotificationType::Admin),
            "general" => Ok(NotificationType::General),
            "marketing" => Ok(NotificationType::Marketing),
            other => Err(format!("Unrecognized enum variant: {}", other).into()),
        }
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            NotificationType::Info => "info",
            NotificationType::Warning => "warning",
            NotificationType::Error => "error", 
            NotificationType::Success => "success",
            NotificationType::Feature => "feature",
            NotificationType::Security => "security",
            NotificationType::System => "system",
            NotificationType::Admin => "admin",
            NotificationType::General => "general",
            NotificationType::Marketing => "marketing",
        };
        write!(f, "{}", value)
    }
}

impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg> for NotificationPriority {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        let value = match self {
            NotificationPriority::Low => "low",
            NotificationPriority::Normal => "normal",
            NotificationPriority::High => "high",
            NotificationPriority::Critical => "critical",
            NotificationPriority::Urgent => "urgent",
        };
        <str as diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg>>::to_sql(value, out)
    }
}

impl diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for NotificationPriority {
    fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match <String as diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg>>::from_sql(bytes)?.as_str() {
            "low" => Ok(NotificationPriority::Low),
            "normal" => Ok(NotificationPriority::Normal),
            "high" => Ok(NotificationPriority::High),
            "critical" => Ok(NotificationPriority::Critical),
            "urgent" => Ok(NotificationPriority::Urgent),
            other => Err(format!("Unrecognized enum variant: {}", other).into()),
        }
    }
}

impl std::fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            NotificationPriority::Low => "low",
            NotificationPriority::Normal => "normal",
            NotificationPriority::High => "high",
            NotificationPriority::Critical => "critical",
            NotificationPriority::Urgent => "urgent",
        };
        write!(f, "{}", value)
    }
}

impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg> for DeliveryChannel {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        let value = match self {
            DeliveryChannel::Email => "email",
            DeliveryChannel::Push => "push",
            DeliveryChannel::SMS => "sms",
            DeliveryChannel::Sms => "sms",
            DeliveryChannel::FcmPush => "fcm_push",
            DeliveryChannel::InApp => "in_app",
        };
        <str as diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg>>::to_sql(value, out)
    }
}

impl diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for DeliveryChannel {
    fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match <String as diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg>>::from_sql(bytes)?.as_str() {
            "email" => Ok(DeliveryChannel::Email),
            "push" => Ok(DeliveryChannel::Push),
            "sms" => Ok(DeliveryChannel::SMS),
            "fcm_push" => Ok(DeliveryChannel::FcmPush),
            "in_app" => Ok(DeliveryChannel::InApp),
            other => Err(format!("Unrecognized enum variant: {}", other).into()),
        }
    }
}

impl std::fmt::Display for DeliveryChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            DeliveryChannel::Email => "email",
            DeliveryChannel::Push => "push",
            DeliveryChannel::SMS => "sms",
            DeliveryChannel::Sms => "sms",
            DeliveryChannel::FcmPush => "fcm_push",
            DeliveryChannel::InApp => "in_app",
        };
        write!(f, "{}", value)
    }
}

/// Wrapper for IP address that can be stored in the database as PostgreSQL Inet type
#[derive(Debug, Clone, Serialize, Deserialize, diesel::AsExpression, diesel::FromSqlRow)]
#[diesel(sql_type = diesel::pg::sql_types::Inet)]
pub struct DieselIpAddr(pub std::net::IpAddr);

// For now, let's use a simple string conversion approach that works
impl diesel::serialize::ToSql<diesel::pg::sql_types::Inet, diesel::pg::Pg> for DieselIpAddr {
    fn to_sql(&self, out: &mut diesel::serialize::Output<diesel::pg::Pg>) -> diesel::serialize::Result {
        // Convert IP to string format that PostgreSQL Inet can handle
        let ip_str = self.0.to_string();
        // Use raw bytes approach for Inet
        out.write_all(ip_str.as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl diesel::deserialize::FromSql<diesel::pg::sql_types::Inet, diesel::pg::Pg> for DieselIpAddr {
    fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = std::str::from_utf8(bytes.as_bytes())?;
        // Parse as either plain IP or CIDR notation and extract IP
        let ip_addr = if s.contains('/') {
            // CIDR notation, extract IP part
            let network: ipnetwork::IpNetwork = s.parse()
                .map_err(|e| format!("Invalid IP network: {}", e))?;
            network.ip()
        } else {
            // Plain IP address
            s.parse().map_err(|e| format!("Invalid IP address: {}", e))?
        };
        Ok(DieselIpAddr(ip_addr))
    }
}