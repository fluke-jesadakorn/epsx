use diesel::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::infra::db::diesel::schema::{stocks, eps_growth_rankings};
use crate::infra::db::diesel::types::DieselDecimal;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = stocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselStock {
    pub symbol: String,
    pub name: String,
    pub market: String,
    pub price: DieselDecimal,
    pub volume: i64,
    pub market_cap: Option<DieselDecimal>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = stocks)]
pub struct NewDieselStock {
    pub symbol: String,
    pub name: String,
    pub market: String,
    pub price: DieselDecimal,
    pub volume: i64,
    pub market_cap: Option<DieselDecimal>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = stocks)]
pub struct UpdateDieselStock {
    pub price: DieselDecimal,
    pub volume: i64,
    pub market_cap: Option<DieselDecimal>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = eps_growth_rankings)]
#[diesel(check_for_backend(diesel::pg::Pg))]  
pub struct DieselEpsGrowthRanking {
    pub id: uuid::Uuid,
    pub symbol: String,
    pub company_name: String,
    pub current_eps: DieselDecimal,
    pub previous_eps: DieselDecimal,
    pub eps_growth_rate: DieselDecimal,
    pub market_cap: Option<DieselDecimal>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub rank_position: i32,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = eps_growth_rankings)]
pub struct NewDieselEpsGrowthRanking {
    pub id: uuid::Uuid,
    pub symbol: String,
    pub company_name: String,
    pub current_eps: DieselDecimal,
    pub previous_eps: DieselDecimal,
    pub eps_growth_rate: DieselDecimal,
    pub market_cap: Option<DieselDecimal>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub rank_position: i32,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}