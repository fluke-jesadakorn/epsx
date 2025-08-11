use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // Test direct WebSocket connection like Node.js
    println!("🚀 Testing Direct TradingView WebSocket Connection");
    
    use epsx::infra::services::tradingview_websocket::TradingViewWebSocketService;
    
    let mut ws_service = TradingViewWebSocketService::new();
    
    println!("📊 Starting WebSocket connection for MSFT EPS data...");
    
    let symbols = vec!["MSFT".to_string()];
    match ws_service.connect_and_fetch_eps_data(symbols).await {
        Ok(eps_data) => {
            println!("✅ WebSocket data collection completed - {} entries", eps_data.len());
            
            for eps in eps_data {
                println!("📈 Symbol: {}", eps.symbol);
                println!("💰 Current EPS: {}", eps.current_eps);
                println!("📊 Quarterly data points: {}", eps.quarterly_data.len());
                
                for q in &eps.quarterly_data[..3.min(eps.quarterly_data.len())] {
                    println!("  {} = {}", q.quarter_name, q.eps);
                }
            }
        }
        Err(e) => {
            println!("❌ WebSocket test failed: {:?}", e);
        }
    }
}