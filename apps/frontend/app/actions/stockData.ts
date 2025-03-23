"use server";

import type { TableDataMetrics, TradingViewResponse } from "@/types/stockFetchData";

const formatLargeNumber = (num: number): string => {
  if (num >= 1e12) return `${(num / 1e12).toFixed(2)}T`;
  if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`;
  if (num >= 1e6) return `${(num / 1e6).toFixed(2)}M`;
  return num.toFixed(2);
};

const formatNumber = (num: number): string => {
  if (num === null || isNaN(num)) return "N/A";
  return num.toFixed(2);
};

const formatDate = (timestamp: number): string => {
  if (!timestamp) return "N/A";
  return new Date(timestamp * 1000).toLocaleDateString();
};

const getAnalysisPhases = (
  lastAnalysis: number,
  nextAnalysis: number
): {
  entryPhase: { date: string; active: boolean };
  phaseStatus: { date: string; type: "monitor" | "exit"; active: boolean };
} => {
  const now = Date.now();
  const lastDate = lastAnalysis * 1000;
  const nextDate = nextAnalysis * 1000;

  // Calculate dates
  const entryDate = new Date(lastDate);
  entryDate.setDate(entryDate.getDate() + 1); // Start phase 1 day after last analysis

  const monitorDate = new Date(nextDate);
  monitorDate.setDate(monitorDate.getDate() - 7); // Start monitoring 7 days before next analysis

  return {
    entryPhase: {
      date: entryDate.toLocaleDateString(),
      active: now >= entryDate.getTime() && now < monitorDate.getTime(),
    },
    phaseStatus: {
      date: monitorDate.toLocaleDateString(),
      type: "monitor",
      active: now >= monitorDate.getTime(),
    },
  };
};

export async function fetchStockScreenerData(): Promise<TableDataMetrics[]> {
  const response = await fetch(
    "https://scanner.tradingview.com/global/scan?label-product=underchart-screener-stock",
    {
      method: "POST",
      headers: {
        accept: "text/plain, */*; q=0.01",
        "accept-language": "th-TH,th;q=0.9,en;q=0.8",
        "content-type": "application/x-www-form-urlencoded; charset=UTF-8",
        origin: "https://www.tradingview.com",
        referer: "https://www.tradingview.com/",
      },
      body: JSON.stringify({
        filter: [
          { left: "High.All", operation: "eless", right: "high" },
          { left: "is_primary", operation: "equal", right: true },
          { left: "active_symbol", operation: "equal", right: true },
          { left: "basic_eps_net_income", operation: "greater", right: 0 },
          {
            left: "earnings_per_share_diluted_ttm",
            operation: "greater",
            right: 0,
          },
          { left: "last_annual_eps", operation: "greater", right: 0 },
          {
            left: "earnings_per_share_forecast_next_fq",
            operation: "greater",
            right: 0,
          },
          { left: "earnings_per_share_fq", operation: "greater", right: 0 },
          {
            left: "earnings_per_share_diluted_qoq_growth_fq",
            operation: "greater",
            right: 0,
          },
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
        range: [0, 1000], // Show up to 1000 items
      }),
      next: {
        revalidate: 300, // Cache for 5 minutes
        tags: ["stockData"],
      },
    }
  );

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  const data: TradingViewResponse = await response.json();

  return data.data.map((stock) => {
    const analysisPhases = getAnalysisPhases(stock.d[17], stock.d[18]);

    return {
      symbol: stock.s.split(":")[1], // Remove exchange prefix
      name: stock.d[0] || stock.d[1], // Use name if available, fallback to symbol
      valueIndex: formatNumber(stock.d[2]),
      growthRate: formatNumber(stock.d[3]),
      activityScore: formatLargeNumber(stock.d[6]),
      marketSize: formatLargeNumber(stock.d[8]),
      growthFactor: formatNumber(stock.d[9]),
      sector: stock.d[11] || "N/A",
      country: stock.d[12] || "N/A",
      exchange: stock.d[13] || "N/A",
      currency: stock.d[27],
      metricScore: formatNumber(stock.d[10]),
      growthIndicator: formatNumber(stock.d[15]), // Growth QoQ
      currentMetric: formatNumber(stock.d[14]), // Current Quarter Metric
      predictedMetric: formatNumber(stock.d[16]), // Next Quarter Prediction
      lastAnalysisDate: formatDate(stock.d[17]), // Last Analysis Date
      nextAnalysisDate: formatDate(stock.d[18]), // Next Analysis Date
      ...analysisPhases,
    };
  });
}

export async function fetchEpsGrowthRanking(
  params: {
    limit?: number;
    skip?: number;
    sortBy?: "growthIndicator" | "activityScore";
  } = {}
): Promise<{ data: TableDataMetrics[] }> {
  const allData = await fetchStockScreenerData();

  // Set default values
  const skip = params.skip ?? 0;
  const limit = params.limit ?? 10;

  // Sort data based on the sortBy parameter
  const sortedData = allData.sort((a, b) => {
    if (params.sortBy === "activityScore") {
      // Convert formatted strings (e.g., "1.23M") to numbers and sort high to low
      const scoreA = parseFloat(a.activityScore.replace(/[^0-9.]/g, "")) || 0;
      const scoreB = parseFloat(b.activityScore.replace(/[^0-9.]/g, "")) || 0;
      if (scoreB > scoreA) return 1; // b comes first
      if (scoreB < scoreA) return -1; // a comes first
      return 0;
    }
    // Default to growth indicator sorting
    const growthA = parseFloat(a.growthIndicator);
    const growthB = parseFloat(b.growthIndicator);
    return growthB - growthA;
  });

  // Apply pagination
  const paginatedData = sortedData.slice(skip, skip + limit);

  return {
    data: paginatedData.map((item) => ({
      ...item,
      // Ensure growth indicator is formatted as percentage
      growthIndicator: `${item.growthIndicator}%`,
    })),
  };
}
