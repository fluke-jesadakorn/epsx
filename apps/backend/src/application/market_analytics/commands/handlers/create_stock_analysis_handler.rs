use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::market_analytics::commands::{
    CreateStockAnalysisCommand, CreateStockAnalysisResponse
};
use crate::domain::market_analytics::{
    StockAnalysisRepositoryPort, StockAnalysis, StockSymbol, EPSValue, MarketSector, Country
};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for creating stock analyses
pub struct CreateStockAnalysisCommandHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CreateStockAnalysisCommandHandler {
    pub fn new(
        stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            stock_analysis_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateStockAnalysisCommand> for CreateStockAnalysisCommandHandler {
    async fn handle(&self, command: CreateStockAnalysisCommand) -> ApplicationResult<CreateStockAnalysisResponse> {
        // 1. Validate and create value objects
        let symbol = StockSymbol::new(command.symbol.clone())
            .map_err(|e| ApplicationError::validation("symbol", e.to_string()))?;

        let current_eps = EPSValue::new(command.current_eps)
            .map_err(|e| ApplicationError::validation("current_eps", e.to_string()))?;

        let previous_eps = EPSValue::new(command.previous_eps)
            .map_err(|e| ApplicationError::validation("previous_eps", e.to_string()))?;

        let sector = MarketSector::new(command.sector)
            .map_err(|e| ApplicationError::validation("sector", e.to_string()))?;

        let country = Country::new(command.country)
            .map_err(|e| ApplicationError::validation("country", e.to_string()))?;

        // 2. Check if stock analysis already exists
        if self.stock_analysis_repository.symbol_exists(&symbol).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))? {
            return Err(ApplicationError::conflict("Stock analysis already exists for this symbol"));
        }

        // 3. Create stock analysis aggregate (business logic in domain)
        let stock_analysis = StockAnalysis::new(
            symbol.clone(),
            command.company_name.clone(),
            current_eps,
            previous_eps,
            sector,
            country,
        ).map_err(|e| ApplicationError::business_logic(e.to_string()))?;

        // 4. Save stock analysis
        self.stock_analysis_repository.save(&stock_analysis).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 5. Publish domain events
        for event in stock_analysis.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 6. Return response
        Ok(CreateStockAnalysisResponse {
            symbol: command.symbol,
            company_name: command.company_name,
            analysis_score: stock_analysis.analysis_score().overall_score,
            eps_growth: stock_analysis.eps_growth().value(),
            growth_classification: stock_analysis.eps_growth().classify().as_str().to_string(),
            created_at: stock_analysis.last_updated(),
        })
    }
}
