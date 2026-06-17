use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::market_analytics::commands::{
    UpdateStockAnalysisCommand, UpdateStockAnalysisResponse
};
use crate::domain::market_analytics::{
    StockAnalysisRepositoryPort, StockSymbol, EPSValue, MarketSector, Country
};
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// Command handler for updating stock analyses
pub struct UpdateStockAnalysisCommandHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl UpdateStockAnalysisCommandHandler {
    pub fn new(
        stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            stock_analysis_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl CommandHandler<UpdateStockAnalysisCommand> for UpdateStockAnalysisCommandHandler {
    async fn handle(&self, command: UpdateStockAnalysisCommand) -> ApplicationResult<UpdateStockAnalysisResponse> {
        // 1. Validate symbol
        let symbol = StockSymbol::new(command.symbol.clone())
            .map_err(|e| ApplicationError::validation("symbol", e.to_string()))?;

        // 2. Load existing stock analysis
        let mut stock_analysis = self.stock_analysis_repository.find_by_symbol(&symbol).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("symbol", "Stock analysis not found"))?;

        // 3. Update EPS data if provided
        if let (Some(current), Some(previous)) = (command.current_eps, command.previous_eps) {
            let current_eps = EPSValue::new(current)
                .map_err(|e| ApplicationError::validation("current_eps", e.to_string()))?;
            let previous_eps = EPSValue::new(previous)
                .map_err(|e| ApplicationError::validation("previous_eps", e.to_string()))?;

            stock_analysis.update_eps(current_eps, previous_eps)
                .map_err(|e| ApplicationError::business_logic(e.to_string()))?;
        }

        // 4. Update sector if provided
        if let Some(sector_str) = command.sector {
            let sector = MarketSector::new(sector_str)
                .map_err(|e| ApplicationError::validation("sector", e.to_string()))?;
            stock_analysis.update_sector(sector)
                .map_err(|e| ApplicationError::business_logic(e.to_string()))?;
        }

        // 5. Update country if provided
        if let Some(country_str) = command.country {
            let country = Country::new(country_str)
                .map_err(|e| ApplicationError::validation("country", e.to_string()))?;
            stock_analysis.update_country(country)
                .map_err(|e| ApplicationError::business_logic(e.to_string()))?;
        }

        // 6. Save updated stock analysis
        self.stock_analysis_repository.save(&stock_analysis).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 7. Publish domain events
        for event in stock_analysis.uncommitted_events() {
            let owned: Box<dyn crate::domain::shared_kernel::DomainEvent> = Box::new(epsx_contracts::domain_event::OwnedEvent::from_borrowed(&**event));
            if let Err(e) = self.event_publisher.publish(owned).await {
                tracing::warn!(
                    error = %e,
                    "EventPublisherPort.publish returned error; command continues"
                );
            }
        }

        // 8. Return response
        Ok(UpdateStockAnalysisResponse {
            symbol: command.symbol,
            analysis_score: stock_analysis.analysis_score().overall_score,
            eps_growth: stock_analysis.eps_growth().value(),
            updated_at: stock_analysis.last_updated(),
        })
    }
}
