use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::infra::db::diesel::schema::payments;
use crate::infra::db::diesel::types::DieselDecimal;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = payments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = payments)]
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

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = payments)]
pub struct UpdateDieselPayment {
    pub status: Option<String>,
    pub payment_method: Option<String>,
    pub transaction_id: Option<String>,
    pub updated_at: DateTime<Utc>,
}