use chrono::{NaiveDate, Utc};

fn main() {
    let test_dates = vec![
        "2025-10-28",  // MSFT
        "2025-10-23",  // AMZN
        "2025-10-22",  // META
        "2025-10-15",  // TSLA
        "2025-10-30",  // From debug file (LLY)
    ];
    
    let current_date = Utc::now();
    let current_naive = current_date.naive_utc().date();
    let current_datetime = current_naive.and_hms_opt(0, 0, 0).unwrap();
    
    println!("Current date: {}", current_date.format("%Y-%m-%d"));
    println!("Testing date parsing and days calculation:\n");
    
    for date_str in test_dates {
        let parsed = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        
        match parsed {
            Ok(parsed_date) => {
                let target_date = parsed_date.and_hms_opt(0, 0, 0).unwrap();
                let days = (target_date - current_datetime).num_days();
                println!("✅ {} -> {} days until announcement", date_str, days);
            }
            Err(e) => {
                println!("❌ {} -> Parse failed: {}", date_str, e);
            }
        }
    }
    
    // Test the actual format from backend (YYYY-MM-DD)
    println!("\nTesting actual backend format:");
    let backend_date = "2025-10-30";
    if let Ok(parsed) = NaiveDate::parse_from_str(backend_date, "%Y-%m-%d") {
        let target = parsed.and_hms_opt(0, 0, 0).unwrap();
        let days = (target - current_datetime).num_days();
        println!("Backend format '{}' -> {} days", backend_date, days);
    }
}