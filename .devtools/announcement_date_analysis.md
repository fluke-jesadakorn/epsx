# Announcement Date Data Flow Analysis

## Debug Session Results

**Date:** August 27, 2025  
**Session Duration:** 2 hours  
**Approach:** Added comprehensive debug logging to trace announcement date data flow

## Summary of Findings

###  **Raw Data Availability**
- **TradingView WebSocket DOES provide announcement dates**: Confirmed in logs
  - MSFT 2025-Q3: announcement timestamp `1753833600`
  - META 2025-Q3: announcement timestamp `1753833600` 
  - TSLA 2025-Q3: announcement timestamp `1753277400`
- **WebSocket service extracts dates correctly**: `estimated_earnings_date` field populated
- **Quarter data contains announcement timestamps**: Present in `QuarterlyEPSData` structure

### = **Data Loss Analysis** 

#### **Root Cause Identified: Transform Layer Data Loss**

The announcement date information is being **lost in the transform layer**. Specifically:

**Location:** `/apps/backend/src/web/analytics/eps/transform.rs` lines 228-237

```rust
result.push(QuarterlyData {
    quarter: quarter_data.quarter_name.clone(), // ê Uses quarter name like "2025-Q3"
    date: chrono::DateTime::<chrono::Utc>::from_timestamp(quarter_data.timestamp, 0), // ê Uses earnings timestamp, not announcement
    price: adjusted_price,
    eps: quarterly_eps,
    eps_growth,
    price_growth,
    volume: ranking.volume.map(|v| ((v as f64) * (1.0 - i as f64 * 0.1).max(0.5)) as i64),
});
```

**Problem:** The `QuarterlyData` struct uses:
- `quarter: quarter_data.quarter_name` í "2025-Q3" (generic)
- `date: quarter_data.timestamp` í Earnings date, not announcement date

**Lost Data:** `quarter_data.estimated_earnings_date` (announcement timestamp) is never used!

### =  **Data Flow Confirmed**

1. ** WebSocket Service**: Successfully extracts `estimated_earnings_date` 
2. ** Enhancement Layer**: Passes WebSocket data to transform 
3. **L Transform Layer**: **DROPS announcement date**, uses generic quarter labels
4. ** DTO Layer**: Correctly serializes what it receives (generic quarters)
5. ** Frontend**: Displays exactly what backend sends

### <Ø **Exact Fix Required**

#### **Backend Changes Needed:**

1. **Update DTO Structure** (`/apps/backend/src/web/analytics/eps/dto.rs`):
```rust
pub struct QuarterlyPerformanceData {
    pub quarter: String,      // Keep for backward compatibility
    pub date: String,         
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64,
    pub price_growth: f64,
    // NEW FIELDS:
    pub announcement_date: Option<String>,     // "Est. Oct 24, 2025"
    pub announcement_timestamp: Option<i64>,   // Raw timestamp
    pub is_estimated: bool,                    // Future vs past
}
```

2. **Update Transform Logic** (`/apps/backend/src/web/analytics/eps/transform.rs`):
```rust
let (announcement_text, is_estimated) = if let Some(announcement_ts) = quarter_data.estimated_earnings_date {
    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(announcement_ts, 0)
        .unwrap_or_default();
    let now = chrono::Utc::now();
    
    let formatted_date = dt.format("%b %-d, %Y").to_string();
    let is_future = dt > now;
    
    if is_future {
        (format!("Est. {}", formatted_date), true)
    } else {
        (format!("Announced {}", formatted_date), false)
    }
} else {
    (quarter_data.quarter_name.clone(), false) // Fallback to quarter
};

QuarterlyPerformanceData {
    quarter: announcement_text,  // Use announcement date instead of "Q3 2025"
    date: dt.format("%b %-d, %Y").to_string(),
    // ... other fields
    announcement_date: Some(announcement_text),
    announcement_timestamp: quarter_data.estimated_earnings_date,
    is_estimated,
}
```

#### **Frontend Changes (Optional):**
Frontend is already compatible - it displays whatever `quarter` field contains.

### =Ä **Expected Results After Fix**
- Cards will show "**Est. Oct 24, 2025**" instead of "2025-Q3"
- Cards will show "**Announced Jul 25, 2024**" instead of "2024-Q2"
- More informative timeline for users
- Backward compatible with existing frontend code

## Debug Files Generated
- `price_growth_debug.log`:  Transform pipeline working 
- `function_calls.log`:  Function calls traced
- `websocket_raw_data.json`: L Empty (file permission issue)
- `transform_pipeline.log`: L Empty (file permission issue)

**Note:** Debug file logging had permission issues, but console logs provided sufficient evidence.

## Confidence Level: **HIGH** <Ø
This analysis is based on direct observation of:
1. WebSocket logs showing announcement timestamps
2. Transform code review showing data loss location
3. API response analysis showing generic quarter labels
4. Frontend compatibility confirmed

**Ready to implement fix with high confidence of success.**