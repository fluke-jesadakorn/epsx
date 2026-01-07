// Get Sectors By Country Query

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;
use crate::application::market_analytics::dtos::SectorsResponse;

#[derive(Debug, Clone)]
pub struct GetSectorsByCountryQuery {
    pub country: Option<String>,
}

impl Query for GetSectorsByCountryQuery {
    type Response = GetSectorsByCountryResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSectorsByCountryResponse {
    pub sectors: SectorsResponse,
}
