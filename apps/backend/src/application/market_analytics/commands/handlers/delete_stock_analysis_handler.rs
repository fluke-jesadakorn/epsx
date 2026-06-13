use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::market_analytics::commands::{
    DeleteStockAnalysisCommand, DeleteStockAnalysisResponse
};
use crate::domain::market_analytics::{StockAnalysisRepositoryPort, StockSymbol};
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// Command handler for deleting stock analyses
pub struct DeleteStockAnalysisCommandHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl DeleteStockAnalysisCommandHandler {
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
impl CommandHandler<DeleteStockAnalysisCommand> for DeleteStockAnalysisCommandHandler {
    async fn handle(&self, command: DeleteStockAnalysisCommand) -> ApplicationResult<DeleteStockAnalysisResponse> {
        // 1. Validate symbol
        let symbol = StockSymbol::new(command.symbol.clone())
            .map_err(|e| ApplicationError::validation("symbol", e.to_string()))?;

        // 2. Check if stock analysis exists
        let stock_analysis = self.stock_analysis_repository.find_by_symbol(&symbol).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("symbol", "Stock analysis not found"))?;

        // 3. Delete stock analysis
        self.stock_analysis_repository.delete(&symbol).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Publish domain events (from aggregate before deletion)
        for event in stock_analysis.uncommitted_events() {
            let owned: Box<dyn crate::domain::shared_kernel::DomainEvent> = Box::new(epsx_contracts::domain_event::OwnedEvent::from_borrowed(&**event));
            if let Err(e) = self.event_publisher.publish(owned).await {
                tracing::warn!(
                    error = %e,
                    "EventPublisherPort.publish returned error; command continues"
                );
            }
        }

        // 5. Return response
        Ok(DeleteStockAnalysisResponse {
            symbol: command.symbol,
            deleted_at: Utc::now(),
        })
    }
}
