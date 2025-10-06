// Get EPS Service Health Query

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;
use crate::application::trading_analytics::dtos::EPSHealthResponse;

#[derive(Debug, Clone)]
pub struct GetEPSServiceHealthQuery {}

impl Query for GetEPSServiceHealthQuery {
    type Response = GetEPSServiceHealthResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEPSServiceHealthResponse {
    pub health: EPSHealthResponse,
}
