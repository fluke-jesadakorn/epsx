use epsx::infra::services::tradingview_websocket::TradingViewWebSocketService;
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Testing Direct TradingView WebSocket Extraction");
    
    let mut ws_service = TradingViewWebSocketService::new();
    
    info!("📊 Starting WebSocket connection for NVDA EPS data...");
    
    let symbols = vec!["NVDA".to_string()];
    match ws_service.connect_and_fetch_eps_data(symbols).await {
        Ok(eps_data) => {
            info!("✅ WebSocket extraction completed - {} entries", eps_data.len());
            
            for eps in eps_data {
                println!("========================================");
                println!("📈 Symbol: {}", eps.symbol);
                println!("💰 Current EPS: {}", eps.current_eps);
                println!("📊 Quarterly data points: {}", eps.quarterly_data.len());
                println!("💲 Price data points: {}", eps.price_data.len());
                println!("========================================");
                
                println!("📊 EPS Progression:");
                for (i, q) in eps.quarterly_data.iter().enumerate() {
                    if i < 8 { // Show first 8 quarters
                        println!("  {} = ${:.2}", q.quarter_name, q.eps);
                    }
                }
                
                if !eps.price_data.is_empty() {
                    println!("💲 Latest Price: ${:.2}", eps.price_data.last().unwrap().close);
                }
            }
        }
        Err(e) => {
            println!("❌ WebSocket extraction failed: {:?}", e);
        }
    }
}