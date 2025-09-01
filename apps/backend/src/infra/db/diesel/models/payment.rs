
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};


// use crate::infra::db::diesel::schema::payments; // Table not in schema
use crate::infra::db::diesel::types::DieselDecimal;


// Disabled - payments table not in schema
// #[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
// #[diesel(table_name = payments)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DieselPayment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: DieselDecimal,
    pub currency: String,
    pub status: String,
    pub payment_method: Option<String>,
    pub transaction_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
// #[diesel(table_name = payments)] // Table not in schema
pub struct NewDieselPayment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: DieselDecimal,
    pub currency: String,
    pub status: String,
    pub payment_method: Option<String>,
    pub transaction_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
// #[diesel(table_name = payments)] // Table not in schema
pub struct UpdateDieselPayment {
    pub status: Option<String>,
    pub payment_method: Option<String>,
    pub transaction_id: Option<String>,
    pub updated_at: DateTime<Utc>,
}