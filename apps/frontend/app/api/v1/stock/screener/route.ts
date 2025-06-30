import { NextResponse } from 'next/server';
import { getFirestoreAdmin } from '../../../../../lib/firebase-admin';
import type { TableDataMetrics } from '../../../../../types/stockFetchData';

// Firestore collection name for stock screener data
const COLLECTION_NAME = 'stockScreenerData';

// Cache duration in seconds (5 minutes)
const CACHE_DURATION = 300;

export async function GET() {
  try {
    // Initialize Firestore
    const db = getFirestoreAdmin();

    // Check for cached data in Firestore
    const cacheRef = db.collection(COLLECTION_NAME).doc('latest');
    const cacheDoc = await cacheRef.get();

    const now = Date.now() / 1000; // Current time in seconds

    if (cacheDoc.exists) {
      const cachedData = cacheDoc.data();
      if (cachedData) {
        const cacheTimestamp = cachedData.timestamp || 0;

        // Check if cached data is still valid (within 5 minutes)
        if (now - cacheTimestamp < CACHE_DURATION) {
          return NextResponse.json(cachedData.data, {
            headers: {
              'Cache-Control': `public, max-age=${CACHE_DURATION}`,
            },
          });
        }
      }
    }

    // If no valid cache, fetch new data from TradingView
    const newData = await fetchStockScreenerDataFromTradingView();

    // Store the new data in Firestore with timestamp
    await cacheRef.set({
      data: newData,
      timestamp: now,
    });

    return NextResponse.json(newData, {
      headers: {
        'Cache-Control': `public, max-age=${CACHE_DURATION}`,
      },
    });
  } catch (error) {
    console.error('Error fetching stock screener data:', error);
    return NextResponse.json(
      { error: 'Failed to fetch stock screener data' },
      { status: 500 }
    );
  }
}

 // Function to fetch stock screener data from TradingView
async function fetchStockScreenerDataFromTradingView(): Promise<TableDataMetrics[]> {
  console.log('Fetching stock screener data from TradingView...');

  // Construct the request payload based on the Rust backend logic
  const requestPayload = {
    filter: [
      { left: "High.All", operation: "eless", right: "high" },
      { left: "is_primary", operation: "equal", right: true },
      { left: "active_symbol", operation: "equal", right: true },
      { left: "basic_eps_net_income", operation: "greater", right: 0 },
      { left: "earnings_per_share_diluted_ttm", operation: "greater", right: 0 },
      { left: "last_annual_eps", operation: "greater", right: 0 },
      { left: "earnings_per_share_forecast_next_fq", operation: "greater", right: 0 },
      { left: "earnings_per_share_fq", operation: "greater", right: 0 },
      { left: "earnings_per_share_diluted_qoq_growth_fq", operation: "greater", right: 0 }
    ],
    options: { lang: "en" },
    markets: [
      "america", "uk", "india", "spain", "russia", "australia", "brazil",
      "japan", "newzealand", "turkey", "switzerland", "hongkong", "taiwan",
      "netherlands", "belgium", "portugal", "france", "mexico", "canada",
      "colombia", "uae", "nigeria", "singapore", "germany", "pakistan",
      "peru", "poland", "italy", "argentina", "israel", "ireland", "egypt",
      "srilanka", "serbia", "chile", "china", "malaysia", "morocco", "ksa",
      "bahrain", "qatar", "indonesia", "finland", "iceland", "denmark",
      "romania", "hungary", "sweden", "slovakia", "lithuania", "luxembourg",
      "estonia", "latvia", "vietnam", "rsa", "thailand", "tunisia", "korea",
      "kenya", "kuwait", "norway", "philippines", "greece", "venezuela",
      "cyprus", "bangladesh", "austria", "czech"
    ],
    symbols: { query: { types: [] }, tickers: [] },
    columns: [
      "logoid", "name", "close", "change", "change_abs", "Recommend.All",
      "volume", "Value.Traded", "market_cap_basic", "price_earnings_ttm",
      "earnings_per_share_basic_ttm", "sector", "country", "exchange",
      "earnings_per_share_fq", "earnings_per_share_diluted_qoq_growth_fq",
      "earnings_per_share_forecast_next_fq", "earnings_release_date",
      "earnings_release_next_date", "description", "type", "subtype",
      "update_mode", "pricescale", "minmov", "fractional", "minmove2",
      "currency", "fundamental_currency_code"
    ],
    sort: { sortBy: "volume", sortOrder: "desc" },
    range: [0, 1000]
  };

  try {
    const response = await fetch("https://scanner.tradingview.com/global/scan", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Accept": "application/json",
        "User-Agent": "Mozilla/5.0 (compatible; EPSX/1.0; +https://epsx.app)"
      },
      body: JSON.stringify(requestPayload)
    });

    if (!response.ok) {
      throw new Error(`TradingView API returned status code: ${response.status}`);
    }

    const data = await response.json();
    // Process the response data to match TableDataMetrics structure
    return processTradingViewResponse(data);
  } catch (error) {
    console.error("Failed to fetch from TradingView:", error);
    throw error;
  }
}

// Helper function to process TradingView response into TableDataMetrics format
function processTradingViewResponse(response: any): TableDataMetrics[] {
  if (!response || !response.data || !Array.isArray(response.data)) {
    console.error("Invalid TradingView response format");
    return [];
  }

  return response.data.map((item: any) => {
    const fields = item.d || [];
    // Map fields to TableDataMetrics structure based on columns requested
    return {
      logoId: fields[0] || "",
      name: fields[1] || "",
      close: fields[2] || 0,
      change: fields[3] || 0,
      changeAbs: fields[4] || 0,
      recommendAll: fields[5] || 0,
      volume: fields[6] || 0,
      valueTraded: fields[7] || 0,
      marketCapBasic: fields[8] || 0,
      priceEarningsTTM: fields[9] || 0,
      earningsPerShareBasicTTM: fields[10] || 0,
      sector: fields[11] || "",
      country: fields[12] || "",
      exchange: fields[13] || "",
      earningsPerShareFQ: fields[14] || 0,
      earningsPerShareDilutedQoQGrowthFQ: fields[15] || 0,
      earningsPerShareForecastNextFQ: fields[16] || 0,
      earningsReleaseDate: fields[17] || "",
      earningsReleaseNextDate: fields[18] || "",
      description: fields[19] || "",
      type: fields[20] || "",
      subtype: fields[21] || "",
      updateMode: fields[22] || "",
      priceScale: fields[23] || 0,
      minMov: fields[24] || 0,
      fractional: fields[25] || false,
      minMove2: fields[26] || 0,
      currency: fields[27] || "",
      fundamentalCurrencyCode: fields[28] || ""
    };
  });
}
