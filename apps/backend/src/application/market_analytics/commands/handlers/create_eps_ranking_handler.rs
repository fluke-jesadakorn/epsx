use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::market_analytics::commands::{
    CreateEPSRankingCommand, CreateEPSRankingResponse, RankingFilters
};
use crate::domain::market_analytics::{
    EPSRankingRepositoryPort, EPSRanking, RankingType, RankingPeriod, SectorCategory, Country
};
// wave11(track-c) R7: kernel-level port for publishing domain events.
// See `epsx_contracts::event_publisher_port` for the design notes.
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// Command handler for creating EPS rankings
pub struct CreateEPSRankingCommandHandler {
    ranking_repository: Arc<dyn EPSRankingRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl CreateEPSRankingCommandHandler {
    pub fn new(
        ranking_repository: Arc<dyn EPSRankingRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            ranking_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateEPSRankingCommand> for CreateEPSRankingCommandHandler {
    async fn handle(&self, command: CreateEPSRankingCommand) -> ApplicationResult<CreateEPSRankingResponse> {
        // 1. Parse ranking type
        let ranking_type = RankingType::from_str(&command.ranking_type)
            .map_err(|e| ApplicationError::validation("ranking_type", e.to_string()))?;

        // 2. Parse time period
        let time_period = RankingPeriod::from_str(&command.time_period)
            .map_err(|e| ApplicationError::validation("time_period", e.to_string()))?;

        // 3. Parse optional filters
        let sector_filter = if let Some(sector_str) = command.sector_filter.as_ref() {
            Some(SectorCategory::from_str(sector_str)
                .map_err(|e| ApplicationError::validation("sector_filter", e.to_string()))?)
        } else {
            None
        };

        let country_filter = if let Some(country_str) = command.country_filter.as_ref() {
            Some(Country::new(country_str.clone())
                .map_err(|e| ApplicationError::validation("country_filter", e.to_string()))?)
        } else {
            None
        };

        // 4. Create EPS ranking aggregate
        let ranking = EPSRanking::new(
            ranking_type.clone(),
            time_period,
            sector_filter,
            country_filter.clone(),
        );

        // 5. Save ranking
        self.ranking_repository.save(&ranking).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Publish domain events via the new `EventPublisherPort` (R7).
        //    See `create_payment_command.rs` for the OwnedEvent
        //    wrapper rationale.
        for event in ranking.uncommitted_events() {
            let owned: Box<dyn crate::domain::shared_kernel::DomainEvent> =
                Box::new(epsx_contracts::domain_event::OwnedEvent::from_borrowed(&**event));
            if let Err(e) = self.event_publisher.publish(owned).await {
                tracing::warn!(
                    error = %e,
                    "EventPublisherPort.publish returned error; command continues"
                );
            }
        }

        // 7. Return response
        Ok(CreateEPSRankingResponse {
            ranking_id: ranking.ranking_id().to_string(),
            ranking_type: command.ranking_type,
            time_period: command.time_period,
            filters: RankingFilters {
                sector: command.sector_filter,
                country: command.country_filter,
            },
            created_at: ranking.last_updated(),
        })
    }
}
