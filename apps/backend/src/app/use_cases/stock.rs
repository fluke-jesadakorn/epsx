// Stock use cases

use std::sync::Arc;

use crate::dom::entities::Stock;
use crate::dom::values::{Symbol, Market};
use crate::app::ports::repositories::StockRepository;
use crate::app::ports::services::{StockDataSvc, WebSocketSvc};
use crate::app::dtos::{GetStockReq, GetStockRes, GetStocksReq, GetStocksRes, SearchStocksReq, SearchStocksRes, TopMoversReq, TopMoversRes, StockDto, StockSearchResult, PriceHistoryReq, PriceHistoryRes, PricePointDto};

pub struct StockUC {
    stock_repo: Arc<dyn StockRepository>,
    stock_data_svc: Arc<dyn StockDataSvc>,
    ws_svc: Arc<dyn WebSocketSvc>,
}

impl StockUC {
    pub fn new(
        stock_repo: Arc<dyn StockRepository>,
        stock_data_svc: Arc<dyn StockDataSvc>,
        ws_svc: Arc<dyn WebSocketSvc>,
    ) -> Self {
        Self {
            stock_repo,
            stock_data_svc,
            ws_svc,
        }
    }
    
    pub async fn get_stock(&self, req: GetStockReq) -> Result<GetStockRes, StockUseCaseError> {
        req.validate().map_err(|e| StockUseCaseError::ValidationError(e.to_string()))?;
        
        let symbol = Symbol::new(&req.sym)
            .map_err(|_| StockUseCaseError::InvalidSymbol(req.sym.clone()))?;
        
        // Try to get from repository first
        if let Ok(Some(stock)) = self.stock_repo.get(&symbol).await {
            if stock.is_recent(15) { // Data is less than 15 minutes old
                return Ok(GetStockRes {
                    stock: StockDto::from_entity(&stock),
                });
            }
        }
        
        // Get fresh data from external service
        let price_data = self.stock_data_svc.get_real_time_price(&symbol).await
            .map_err(|e| StockUseCaseError::ExternalServiceError(e.to_string()))?;
        
        // Create stock entity and save
        let stock = Stock::new(
            symbol,
            price_data.price,
            price_data.volume,
            Market::NASDAQ // TODO: Determine market from symbol
        );
        
        self.stock_repo.save(&stock).await
            .map_err(|e| StockUseCaseError::RepositoryError(e.to_string()))?;
        
        Ok(GetStockRes {
            stock: StockDto::from_entity(&stock),
        })
    }
    
    pub async fn get_stocks(&self, req: GetStocksReq) -> Result<GetStocksRes, StockUseCaseError> {
        req.validate().map_err(|e| StockUseCaseError::ValidationError(e.to_string()))?;
        
        let mut stocks = Vec::new();
        
        for sym_str in &req.syms {
            let symbol = Symbol::new(sym_str)
                .map_err(|_| StockUseCaseError::InvalidSymbol(sym_str.clone()))?;
            
            if let Ok(Some(stock)) = self.stock_repo.get(&symbol).await {
                stocks.push(StockDto::from_entity(&stock));
            }
        }
        
        Ok(GetStocksRes { stocks })
    }
    
    pub async fn search_stocks(&self, req: SearchStocksReq) -> Result<SearchStocksRes, StockUseCaseError> {
        req.validate().map_err(|e| StockUseCaseError::ValidationError(e.to_string()))?;
        
        let symbols = self.stock_data_svc.search_symbols(&req.query).await
            .map_err(|e| StockUseCaseError::ExternalServiceError(e.to_string()))?;
        
        let results = symbols.into_iter()
            .take(req.limit.unwrap_or(50) as usize)
            .map(|symbol_info| StockSearchResult {
                sym: symbol_info.symbol.to_string(),
                name: symbol_info.name,
                market: symbol_info.market,
                sector: symbol_info.sector,
                px: None, // Could be filled from cache
            })
            .collect();
        
        Ok(SearchStocksRes { results })
    }
    
    pub async fn get_top_movers(&self, req: TopMoversReq) -> Result<TopMoversRes, StockUseCaseError> {
        req.validate().map_err(|e| StockUseCaseError::ValidationError(e.to_string()))?;
        
        let stocks = self.stock_repo.find_top_movers(req.limit).await
            .map_err(|e| StockUseCaseError::RepositoryError(e.to_string()))?;
        
        // TODO: Sort by mover_type (gainers, losers, volume)
        
        Ok(TopMoversRes {
            stocks: stocks.iter().map(StockDto::from_entity).collect(),
        })
    }
    
    pub async fn get_price_history(&self, req: PriceHistoryReq) -> Result<PriceHistoryRes, StockUseCaseError> {
        req.validate().map_err(|e| StockUseCaseError::ValidationError(e.to_string()))?;
        
        let symbol = Symbol::new(&req.sym)
            .map_err(|_| StockUseCaseError::InvalidSymbol(req.sym.clone()))?;
        
        let historical_data = self.stock_data_svc.get_historical_data(&symbol, &req.period).await
            .map_err(|e| StockUseCaseError::ExternalServiceError(e.to_string()))?;
        
        let prices = historical_data.into_iter()
            .map(|price_data| PricePointDto {
                px: price_data.price,
                vol: price_data.volume,
                ts: price_data.timestamp,
            })
            .collect();
        
        Ok(PriceHistoryRes {
            sym: req.sym,
            prices,
        })
    }
    
    pub async fn broadcast_price_update(&self, symbol: &Symbol, price: rust_decimal::Decimal) -> Result<(), StockUseCaseError> {
        self.ws_svc.broadcast_stock_update(symbol, price).await
            .map_err(|e| StockUseCaseError::WebSocketError(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StockUseCaseError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    
    #[error("Stock not found: {0}")]
    StockNotFound(String),
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}