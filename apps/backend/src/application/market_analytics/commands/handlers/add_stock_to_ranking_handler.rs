use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::market_analytics::commands::{
    AddStockToRankingCommand, AddStockToRankingResponse
};
use crate::domain::market_analytics::{
    EPSRankingRepositoryPort, StockSymbol, EPSValue, GrowthFactor, MarketSector, Country
};
use epsx_contracts::traits::DomainEventBus;

/// Command handler for adding stocks to rankings
pub struct AddStockToRankingCommandHandler {
    ranking_repository: Arc<dyn EPSRankingRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl AddStockToRankingCommandHandler {
    pub fn new(
        ranking_repository: Arc<dyn EPSRankingRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            ranking_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<AddStockToRankingCommand> for AddStockToRankingCommandHandler {
    async fn handle(&self, command: AddStockToRankingCommand) -> ApplicationResult<AddStockToRankingResponse> {
        // 1. Load ranking
        let mut ranking = self.ranking_repository.find_by_id(&command.ranking_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("ranking_id", "Ranking not found"))?;

        // 2. Validate and create value objects
        let symbol = StockSymbol::new(command.symbol.clone())
            .map_err(|e| ApplicationError::validation("symbol", e.to_string()))?;

        let eps_value = EPSValue::new(command.eps_value)
            .map_err(|e| ApplicationError::validation("eps_value", e.to_string()))?;

        let growth_factor = GrowthFactor::new(command.growth_factor)
            .map_err(|e| ApplicationError::validation("growth_factor", e.to_string()))?;

        let sector = MarketSector::new(command.sector)
            .map_err(|e| ApplicationError::validation("sector", e.to_string()))?;

        let country = Country::new(command.country)
            .map_err(|e| ApplicationError::validation("country", e.to_string()))?;

        // 3. Add stock to ranking (domain logic validates filters)
        let rank = ranking.add_entry(
            symbol.clone(),
            command.company_name,
            eps_value,
            growth_factor,
            sector,
            country,
        ).map_err(|e| ApplicationError::business_logic(e.to_string()))?;

        // 4. Calculate score (from ranking entry)
        let score = ranking.entries().get(&rank)
            .map(|entry| entry.score)
            .unwrap_or(0.0);

        // 5. Save updated ranking
        self.ranking_repository.save(&ranking).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Publish domain events
        for event in ranking.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 7. Return response
        Ok(AddStockToRankingResponse {
            ranking_id: command.ranking_id,
            symbol: command.symbol,
            rank,
            score,
            added_at: Utc::now(),
        })
    }
}
