use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};


use crate::infra::db::diesel::schema::sessions;

use crate::infra::db::diesel::types::DieselIpAddr;


#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<DieselIpAddr>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = sessions)]
pub struct NewDieselSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<DieselIpAddr>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = sessions)]
pub struct UpdateDieselSession {
    pub access_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub session_token: Option<String>,
    pub is_active: Option<bool>,
}