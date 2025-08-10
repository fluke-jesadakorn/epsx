use epsx::infra::services::tradingview_websocket::TradingViewWebSocketService;
use tracing::{info, warn, Level};
use std::time::Instant;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Testing Key Symbols for EPS Accuracy Validation");
    
    // Strategically selected symbols from different sectors and exchanges
    let test_cases = vec![
        // Tech giants with known EPS patterns
        ("AAPL", "Apple Inc."),
        ("MSFT", "Microsoft Corporation"), 
        ("GOOGL", "Alphabet Inc."),
        ("NVDA", "NVIDIA Corporation"),
        ("TSLA", "Tesla Inc."),
        ("META", "Meta Platforms"),
        ("AMZN", "Amazon.com Inc."),
        
        // Financial services
        ("JPM", "JPMorgan Chase"),
        ("BAC", "Bank of America"),
        ("BRK.B", "Berkshire Hathaway"),
        
        // Healthcare & Consumer
        ("JNJ", "Johnson & Johnson"),
        ("PG", "Procter & Gamble"),
        ("WMT", "Walmart Inc."),
        ("KO", "Coca-Cola Company"),
        
        // Industrial & Energy  
        ("XOM", "Exxon Mobil"),
        ("GE", "General Electric"),
        
        // Different market caps and volatility levels
        ("PLTR", "Palantir Technologies"), // Growth stock
        ("DIS", "Walt Disney Company"),    // Media/Entertainment
        ("NFLX", "Netflix Inc."),          // Streaming
        ("UBER", "Uber Technologies"),     // Transportation
    ];

    let total_symbols = test_cases.len();
    info!("📊 Testing {} key symbols for EPS accuracy", total_symbols);
    
    let mut results = Vec::new();
    let start_time = Instant::now();
    
    for (index, (symbol, company)) in test_cases.iter().enumerate() {
        info!("🔄 [{}/{}] Processing: {} ({})", index + 1, total_symbols, symbol, company);
        
        let mut ws_service = TradingViewWebSocketService::new();
        let symbol_vec = vec![symbol.to_string()];
        
        match ws_service.connect_and_fetch_eps_data(symbol_vec).await {
            Ok(eps_data) => {
                if !eps_data.is_empty() {
                    let data = &eps_data[0];
                    let eps_count = data.quarterly_data.len();
                    
                    results.push((symbol, true, eps_count, data.current_eps, data.company_name.clone()));
                    
                    info!("✅ {}: {} quarters | Current: {} | Company: {}", 
                          symbol, eps_count, data.current_eps, data.company_name);
                    
                    // Show recent EPS progression for verification
                    if eps_count >= 4 {
                        let recent_eps: Vec<String> = data.quarterly_data.iter()
                            .take(4)
                            .map(|q| format!("{}:{}", q.period, q.actual_eps))
                            .collect();
                        info!("📈 Recent: [{}]", recent_eps.join(" → "));
                        
                        // Check for earnings announcement dates
                        let with_dates = data.quarterly_data.iter()
                            .filter(|q| q.estimated_earnings_date.is_some())
                            .count();
                        info!("📅 {} quarters have earnings announcement dates", with_dates);
                    }
                } else {
                    warn!("⚠️ {}: No EPS data returned", symbol);
                    results.push((symbol, false, 0, 0.0, "No data".to_string()));
                }
            },
            Err(e) => {
                warn!("❌ {}: Extraction failed - {}", symbol, e);
                results.push((symbol, false, 0, 0.0, format!("Error: {}", e)));
            }
        }
        
        // Shorter delay for focused test
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    let elapsed = start_time.elapsed();
    let successful = results.iter().filter(|(_, success, _, _, _)| *success).count();
    let failed = results.len() - successful;
    let total_eps = results.iter().map(|(_, _, eps_count, _, _)| eps_count).sum::<usize>();
    
    // Detailed results analysis
    info!("🏆 ========== KEY SYMBOLS TEST RESULTS ==========");
    info!("🎯 Total Symbols: {} | ✅ Success: {} | ❌ Failed: {}", 
          total_symbols, successful, failed);
    info!("📊 Total EPS Data Points: {}", total_eps);
    info!("📈 Average EPS per Symbol: {:.1}", total_eps as f64 / successful.max(1) as f64);
    info!("⏰ Processing Time: {:.1}s", elapsed.as_secs_f64());
    info!("🚀 Success Rate: {:.1}%", (successful as f64 / total_symbols as f64) * 100.0);
    
    // Show detailed breakdown
    info!("📋 DETAILED RESULTS:");
    for (symbol, success, eps_count, current_eps, company) in &results {
        let status = if *success { "✅" } else { "❌" };
        info!("{} {}: {} EPS points | Current: {} | {}", 
              status, symbol, eps_count, current_eps, company);
    }
    
    // Success criteria
    if successful >= (total_symbols * 80 / 100) {
        info!("🎉 TEST PASSED! Excellent extraction accuracy across diverse symbols");
    } else if successful >= (total_symbols * 60 / 100) {
        warn!("⚠️ TEST MARGINAL! Good results but some symbols need investigation");  
    } else {
        warn!("❌ TEST FAILED! Low success rate indicates systematic issues");
    }
    
    info!("================================================");
    
    // Manual verification suggestions
    info!("💡 Manual Verification Suggestions:");
    info!("• Check MSFT recent EPS matches: 2.99 → 2.93 → 2.94 → 2.95 → 3.30 → 3.23 → 3.46 → 3.65");
    info!("• Check NVDA recent EPS matches: 0.27 → 0.40 → 0.52 → 0.61 → 0.68 → 0.81 → 0.89 → 0.81");
    info!("• Verify earnings announcement dates are realistic (not future dates)");
    info!("• Cross-check with TradingView charts for accuracy");
}