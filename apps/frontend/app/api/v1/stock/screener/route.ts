import { NextResponse } from 'next/server';
import type { TableDataMetrics } from '../../../../../types/stockFetchData';

// Cache duration in seconds (5 minutes)
const CACHE_DURATION = 300;

export async function GET() {
  try {
    // Fetch data directly from TradingView
    const rawData = await fetchStockScreenerDataFromTradingView();
    // Process TradingView response into TableDataMetrics[]
    const processedData = processTradingViewResponse(rawData);
    return NextResponse.json(processedData, {
      headers: {
        'Cache-Control': `public, max-age=${CACHE_DURATION}`,
      },
    });
  } catch (error) {
    console.error('Error fetching stock screener data:', error);
    return NextResponse.json(
      { error: 'Failed to fetch stock screener data' },
      { status: 500 },
    );
  }
}

// Function to fetch stock screener data from TradingView (using latest user curl)
async function fetchStockScreenerDataFromTradingView(): Promise<any> {
  const url =
    'https://scanner.tradingview.com/global/scan?label-product=screener-stock';
  const headers = {
    accept: 'application/json',
    'accept-language': 'th-TH,th;q=0.9,en;q=0.8',
    'cache-control': 'no-cache',
    'content-type': 'text/plain;charset=UTF-8',
    origin: 'https://www.tradingview.com',
    pragma: 'no-cache',
    priority: 'u=1, i',
    referer: 'https://www.tradingview.com/',
    'sec-ch-ua':
      '"Google Chrome";v="137", "Chromium";v="137", "Not/A)Brand";v="24"',
    'sec-ch-ua-mobile': '?0',
    'sec-ch-ua-platform': '"macOS"',
    'sec-fetch-dest': 'empty',
    'sec-fetch-mode': 'cors',
    'sec-fetch-site': 'same-site',
    'user-agent':
      'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36',
    cookie:
      'cookiePrivacyPreferenceBannerProduction=notApplicable; cookiesSettings={"analytics":true,"advertising":true}; _ga=GA1.1.287998567.1737519655; device_t=Yk1Fb0FROjI.o614l3RjSdzzD_nBlfv7bHmRO9SLuSg8Jdc5C8_Cfyg; sessionid=8muioxxaoyz3d8lvyqey7wa6fnmhzx4f; sessionid_sign=v3:snfqsmVt+TvwXS43RnT/4MEx8OEh36wZ7gWDGEFK+BY=; tv_ecuid=e538a49d-5181-4179-b95f-35fc057358a6; __gads=ID=a2dce82dea5f6034:T=1741599088:RT=1746415506:S=ALNI_MZVTo4XKZH5jxKH9m9ofmqw5e75WQ; __gpi=UID=0000105bb6af1052:T=1741599088:RT=1746415506:S=ALNI_MZae1ihNv6rQ0C6ewUUoX6UxQjcgg; __eoi=ID=c798787be475c469:T=1741599088:RT=1746415506:S=AA-AfjbGRWxAkTkVM-VaX35P2o4H; _sp_ses.cf1a=*; _ga_YVVRYGL0E0=GS2.1.s1751613217$o288$g1$t1751613520$j17$l0$h0; _sp_id.cf1a=b0388acf-1e28-423e-9d8e-0d5a55fad2e9.1737519655.174.1751613976.1751458367.5e74fd68-55fe-4139-ace6-1be5a74d5be1.745260e6-1d50-4f31-95c7-a46a513bd58e.0a845b52-cdae-451f-aad2-8e18dd873606.1751613216785.44',
  };
  const body = JSON.stringify({
    columns: [
      'name',
      'description',
      'logoid',
      'update_mode',
      'type',
      'typespecs',
      'close',
      'pricescale',
      'minmov',
      'fractional',
      'minmove2',
      'currency',
      'change',
      'volume',
      'relative_volume_10d_calc',
      'market_cap_basic',
      'fundamental_currency_code',
      'price_earnings_ttm',
      'earnings_per_share_diluted_ttm',
      'earnings_per_share_diluted_yoy_growth_ttm',
      'dividends_yield_current',
      'sector.tr',
      'market',
      'sector',
      // 'valueIndex', // removed
      // 'activityScore', // removed
      // 'growthFactor', // removed
      'recommendation_mark',
      'exchange',
    ],
    filter: [
      {
        left: 'earnings_per_share_diluted_qoq_growth_fq',
        operation: 'greater',
        right: 0,
      },
      { left: 'is_primary', operation: 'equal', right: true },
    ],
    ignore_unknown_fields: false,
    options: { lang: 'en' },
    price_conversion: { to_currency: 'usd' },
    range: [0, 100],
    sort: { sortBy: 'market_cap_basic', sortOrder: 'desc' },
    symbols: {},
    markets: [
      'america',
      'argentina',
      'australia',
      'austria',
      'bahrain',
      'bangladesh',
      'belgium',
      'brazil',
      'canada',
      'chile',
      'china',
      'colombia',
      'cyprus',
      'czech',
      'denmark',
      'egypt',
      'estonia',
      'finland',
      'france',
      'germany',
      'greece',
      'hongkong',
      'hungary',
      'iceland',
      'india',
      'indonesia',
      'ireland',
      'israel',
      'italy',
      'japan',
      'kenya',
      'kuwait',
      'latvia',
      'lithuania',
      'luxembourg',
      'malaysia',
      'mexico',
      'morocco',
      'netherlands',
      'newzealand',
      'nigeria',
      'norway',
      'pakistan',
      'peru',
      'philippines',
      'poland',
      'portugal',
      'qatar',
      'romania',
      'russia',
      'ksa',
      'serbia',
      'singapore',
      'slovakia',
      'rsa',
      'korea',
      'spain',
      'srilanka',
      'sweden',
      'switzerland',
      'taiwan',
      'thailand',
      'tunisia',
      'turkey',
      'uae',
      'uk',
      'venezuela',
      'vietnam',
    ],
    filter2: {
      operator: 'and',
      operands: [
        {
          operation: {
            operator: 'or',
            operands: [
              {
                operation: {
                  operator: 'and',
                  operands: [
                    {
                      expression: {
                        left: 'type',
                        operation: 'equal',
                        right: 'stock',
                      },
                    },
                    {
                      expression: {
                        left: 'typespecs',
                        operation: 'has',
                        right: ['common'],
                      },
                    },
                  ],
                },
              },
              {
                operation: {
                  operator: 'and',
                  operands: [
                    {
                      expression: {
                        left: 'type',
                        operation: 'equal',
                        right: 'stock',
                      },
                    },
                    {
                      expression: {
                        left: 'typespecs',
                        operation: 'has',
                        right: ['preferred'],
                      },
                    },
                  ],
                },
              },
              {
                operation: {
                  operator: 'and',
                  operands: [
                    {
                      expression: {
                        left: 'type',
                        operation: 'equal',
                        right: 'dr',
                      },
                    },
                  ],
                },
              },
              {
                operation: {
                  operator: 'and',
                  operands: [
                    {
                      expression: {
                        left: 'type',
                        operation: 'equal',
                        right: 'fund',
                      },
                    },
                    {
                      expression: {
                        left: 'typespecs',
                        operation: 'has_none_of',
                        right: ['etf'],
                      },
                    },
                  ],
                },
              },
            ],
          },
        },
      ],
    },
  });

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers,
      body,
    });

    if (!response.ok) {
      throw new Error(
        `TradingView API returned status code: ${response.status}`,
      );
    }

    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Failed to fetch from TradingView:', error);
    throw error;
  }
}

// Helper function to process TradingView response into TableDataMetrics format
function processTradingViewResponse(response: any): TableDataMetrics[] {
  if (!response || !response.data || !Array.isArray(response.data)) {
    console.error('Invalid TradingView response format');
    return [];
  }

  console.log('Processing TradingView response:', response.data);

  return response.data.map((item: any) => {
    const fields = item.d || [];
    // Map fields to TableDataMetrics structure based on columns requested
    return {
      logoId: fields[0] || '',
      name: fields[1] || '',
      close: fields[2] || 0,
      change: fields[3] || 0,
      changeAbs: fields[4] || 0,
      recommendAll: fields[5] || 0,
      volume: fields[6] || 0,
      valueTraded: fields[7] || 0,
      marketCapBasic: fields[8] || 0,
      priceEarningsTTM: fields[9] || 0,
      earningsPerShareBasicTTM: fields[10] || 0,
      sector: fields[11] || '',
      country: fields[12] || '',
      exchange: fields[13] || '',
      earningsPerShareFQ: fields[14] || 0,
      earningsPerShareDilutedQoQGrowthFQ: fields[15] || 0,
      earningsPerShareForecastNextFQ: fields[16] || 0,
      earningsReleaseDate: fields[17] || '',
      earningsReleaseNextDate: fields[18] || '',
      description: fields[19] || '',
      type: fields[20] || '',
      subtype: fields[21] || '',
      updateMode: fields[22] || '',
      priceScale: fields[23] || 0,
      minMov: fields[24] || 0,
      fractional: fields[25] || false,
      minMove2: fields[26] || 0,
      currency: fields[27] || '',
      fundamentalCurrencyCode: fields[28] || '',
      // valueIndex, activityScore, growthFactor removed
    };
  });
}
