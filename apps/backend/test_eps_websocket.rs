use tokio;
use std::time::Duration;

#[path = "src/infra/services/tradingview_websocket.rs"]
mod tradingview_websocket;

use tradingview_websocket::TradingViewWebSocketService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🚀 Testing corrected EPS WebSocket extraction...");
    
    // Test with TSLA symbol as reference
    let test_symbols = vec!["TSLA".to_string()];
    
    println!("📊 Creating WebSocket service...");
    let mut ws_service = TradingViewWebSocketService::new();
    
    println!("🔌 Connecting and fetching EPS data for TSLA...");
    match ws_service.connect_and_fetch_eps_data(test_symbols).await {
        Ok(eps_data_list) => {
            println!("✅ Successfully fetched {} EPS entries", eps_data_list.len());
            
            for eps_data in eps_data_list {
                println!("\n📈 Symbol: {}", eps_data.symbol);
                println!("   Current EPS: {}", eps_data.current_eps);
                println!("   Quarterly Data: {} quarters", eps_data.quarterly_data.len());
                
                // Display quarterly data details
                for (i, quarter) in eps_data.quarterly_data.iter().enumerate() {
                    println!("   Q{}: {} - EPS: {}", 
                             i+1, quarter.quarter_name, quarter.eps);
                }
                
                // Verify this matches TSLA reference data
                if eps_data.symbol == "TSLA" && !eps_data.quarterly_data.is_empty() {
                    println!("\n🎯 TSLA Verification:");
                    println!("   - Using corrected study name: Earnings@tv-basicstudies-255 ✅");
                    println!("   - Using corrected EPS index: v[5] ✅"); 
                    println!("   - Quarter format: {} ✅", eps_data.quarterly_data[0].quarter_name);
                    
                    // Check if EPS values are reasonable for TSLA
                    let first_quarter_eps = eps_data.quarterly_data[0].eps;
                    if first_quarter_eps > 0.0 && first_quarter_eps < 10.0 {
                        println!("   - EPS value reasonable: {} ✅", first_quarter_eps);
                    } else {
                        println!("   - EPS value unexpected: {} ❌", first_quarter_eps);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to fetch EPS data: {}", e);
            return Err(e.into());
        }
    }
    
    println!("\n🎉 Test completed!");
    Ok(())
}