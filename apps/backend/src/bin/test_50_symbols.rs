use epsx::infra::services::tradingview_websocket::TradingViewWebSocketService;
use tracing::{info, warn, Level};
use std::time::Instant;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Testing TradingView WebSocket Extraction for 50+ Symbols");
    
    // Comprehensive list of 50+ popular symbols from different sectors and exchanges
    let symbols = vec![
        // Technology Giants
        "AAPL", "MSFT", "GOOGL", "AMZN", "META", "TSLA", "NVDA", "NFLX", "ADBE", "CRM",
        "ORCL", "INTC", "AMD", "QCOM", "AVGO", "NOW", "SNOW", "PLTR", "RBLX", "UBER",
        
        // Financial Services  
        "JPM", "BAC", "WFC", "GS", "MS", "C", "BRK.B", "V", "MA", "PYPL",
        "AXP", "COF", "USB", "PNC", "TFC",
        
        // Healthcare & Pharma
        "JNJ", "PFE", "UNH", "ABBV", "MRK", "LLY", "TMO", "DHR", "ABT", "BMY",
        
        // Consumer & Retail
        "WMT", "HD", "PG", "KO", "PEP", "MCD", "SBUX", "NKE", "COST", "TGT",
        
        // Industrial & Energy
        "MMM", "BA", "CAT", "GE", "XOM", "CVX", "COP", "SLB", "EOG", "MPC",
        
        // Additional sectors for comprehensive testing
        "DIS", "CMCSA", "VZ", "T", "IBM", "CSCO", "HON", "RTX", "LMT", "UPS"
    ];

    let total_symbols = symbols.len();
    info!("📊 Testing {} symbols across all major sectors", total_symbols);
    
    let mut successful_extractions = 0;
    let mut failed_extractions = 0;
    let mut total_eps_data_points = 0;
    let start_time = Instant::now();
    
    for (index, symbol) in symbols.iter().enumerate() {
        info!("🔄 Processing symbol {}/{}: {}", index + 1, total_symbols, symbol);
        
        let mut ws_service = TradingViewWebSocketService::new();
        let symbol_vec = vec![symbol.to_string()];
        
        match ws_service.connect_and_fetch_eps_data(symbol_vec).await {
            Ok(eps_data) => {
                if !eps_data.is_empty() {
                    let data = &eps_data[0];
                    let eps_count = data.quarterly_data.len();
                    
                    successful_extractions += 1;
                    total_eps_data_points += eps_count;
                    
                    info!("✅ {}: {} EPS quarters | Current EPS: {} | Company: {}", 
                          symbol, eps_count, data.current_eps, data.company_name);
                    
                    // Show first few EPS values to verify accuracy
                    if eps_count > 0 {
                        let recent_eps: Vec<String> = data.quarterly_data.iter()
                            .take(4)
                            .map(|q| format!("{}: {}", q.period, q.actual_eps))
                            .collect();
                        info!("📈 Recent EPS: [{}]", recent_eps.join(", "));
                    }
                } else {
                    warn!("⚠️ {}: No EPS data returned", symbol);
                    failed_extractions += 1;
                }
            },
            Err(e) => {
                warn!("❌ {}: Extraction failed - {}", symbol, e);
                failed_extractions += 1;
            }
        }
        
        // Small delay between requests to be respectful to TradingView servers
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
    
    let elapsed = start_time.elapsed();
    
    // Final comprehensive report
    info!("🏆 ========== COMPREHENSIVE TEST RESULTS ==========");
    info!("🎯 Total Symbols Tested: {}", total_symbols);
    info!("✅ Successful Extractions: {}", successful_extractions);
    info!("❌ Failed Extractions: {}", failed_extractions);
    info!("📊 Total EPS Data Points: {}", total_eps_data_points);
    info!("📈 Average EPS Points per Symbol: {:.1}", 
          total_eps_data_points as f64 / successful_extractions.max(1) as f64);
    info!("⏰ Total Processing Time: {:.2}s", elapsed.as_secs_f64());
    info!("🚀 Success Rate: {:.1}%", 
          (successful_extractions as f64 / total_symbols as f64) * 100.0);
    
    if successful_extractions >= (total_symbols as f64 * 0.8) as usize {
        info!("🎉 COMPREHENSIVE TEST PASSED! High success rate achieved.");
    } else {
        warn!("⚠️ COMPREHENSIVE TEST NEEDS IMPROVEMENT! Low success rate.");
    }
    
    info!("=================================================");
}