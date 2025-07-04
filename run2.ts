// Standalone script to fetch and debug print TradingView screener data
// Usage: run with `node run2.ts` (after compiling to JS if needed)

async function main() {
  // You can adjust these values for testing
  const body = {
    minEpsGrowth: 0,
    limit: 100,
    sortBy: 'market_cap_basic',
    sortOrder: 'desc',
  };

  const scannerPayload = {
    columns: [
      "name",
      "description",
      "logoid",
      "update_mode",
      "type",
      "typespecs",
      "close",
      "pricescale",
      "minmov",
      "fractional",
      "minmove2",
      "currency",
      "change",
      "volume",
      "relative_volume_10d_calc",
      "market_cap_basic",
      "fundamental_currency_code",
      "price_earnings_ttm",
      "earnings_per_share_diluted_ttm",
      "earnings_per_share_diluted_yoy_growth_ttm",
      "dividends_yield_current",
      "sector.tr",
      "market",
      "sector",
      "recommendation_mark",
      "exchange"
    ],
    filter: [
      {
        left: "earnings_per_share_diluted_qoq_growth_fq",
        operation: "greater",
        right: 0
      },
      {
        left: "is_primary",
        operation: "equal",
        right: true
      }
    ],
    ignore_unknown_fields: false,
    options: { lang: "en" },
    price_conversion: { to_currency: "usd" },
    range: [0, body.limit || 100],
    sort: {
      sortBy: body.sortBy || "market_cap_basic",
      sortOrder: body.sortOrder || "desc"
    },
    symbols: {},
    markets: [
      "america","argentina","australia","austria","bahrain","bangladesh","belgium","brazil","canada","chile","china","colombia","cyprus","czech","denmark","egypt","estonia","finland","france","germany","greece","hongkong","hungary","iceland","india","indonesia","ireland","israel","italy","japan","kenya","kuwait","latvia","lithuania","luxembourg","malaysia","mexico","morocco","netherlands","newzealand","nigeria","norway","pakistan","peru","philippines","poland","portugal","qatar","romania","russia","ksa","serbia","singapore","slovakia","rsa","korea","spain","srilanka","sweden","switzerland","taiwan","thailand","tunisia","turkey","uae","uk","venezuela","vietnam"
    ],
    filter2: {
      operator: "and",
      operands: [
        {
          operation: {
            operator: "or",
            operands: [
              {
                operation: {
                  operator: "and",
                  operands: [
                    {
                      expression: {
                        left: "type",
                        operation: "equal",
                        right: "stock"
                      }
                    },
                    {
                      expression: {
                        left: "typespecs",
                        operation: "has",
                        right: ["common"]
                      }
                    }
                  ]
                }
              },
              {
                operation: {
                  operator: "and",
                  operands: [
                    {
                      expression: {
                        left: "type",
                        operation: "equal",
                        right: "stock"
                      }
                    },
                    {
                      expression: {
                        left: "typespecs",
                        operation: "has",
                        right: ["preferred"]
                      }
                    }
                  ]
                }
              },
              {
                operation: {
                  operator: "and",
                  operands: [
                    {
                      expression: {
                        left: "type",
                        operation: "equal",
                        right: "dr"
                      }
                    }
                  ]
                }
              },
              {
                operation: {
                  operator: "and",
                  operands: [
                    {
                      expression: {
                        left: "type",
                        operation: "equal",
                        right: "fund"
                      }
                    },
                    {
                      expression: {
                        left: "typespecs",
                        operation: "has_none_of",
                        right: ["etf"]
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
      ]
    }
  };

  console.log('Sending payload:', JSON.stringify(scannerPayload, null, 2));

  try {
    const response = await fetch(
      'https://scanner.tradingview.com/global/scan?label-product=screener-stock',
      {
        method: 'POST',
        headers: {
          accept: 'application/json',
          'content-type': 'text/plain;charset=UTF-8',
          origin: 'https://www.tradingview.com',
          referer: 'https://www.tradingview.com/',
          'user-agent':
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
        },
        body: JSON.stringify(scannerPayload),
      },
    );

    console.log('Fetch status:', response.status);
    
    if (!response.ok) {
      throw new Error(`TradingView API error: ${response.status}`);
    }

    const data = await response.json();
    console.log('Raw data:', JSON.stringify(data, null, 2));

    if (!data.columns || !Array.isArray(data.columns)) {
      console.error('API response missing "columns" property:', data);
      return;
    }

    const processedData = data.data?.map((stock: any) => {
      const columnMap: { [key: string]: number } = {};
      data.columns.forEach((col: string, index: number) => {
        columnMap[col] = index;
      });

      const currentQuarterEps =
        stock.d?.[columnMap['earnings_per_share_fq']] || 0;
      const previousQuarterEps =
        stock.d?.[columnMap['earnings_per_share_fq1']] || 0;
      const twoQuartersAgoEps =
        stock.d?.[columnMap['earnings_per_share_fq2']] || 0;
      const threeQuartersAgoEps =
        stock.d?.[columnMap['earnings_per_share_fq3']] || 0;

      let oneQoQGrowth: number | null = null;
      let twoQoQGrowth: number | null = null;
      if (previousQuarterEps !== 0) {
        oneQoQGrowth =
          ((currentQuarterEps - previousQuarterEps) /
            Math.abs(previousQuarterEps)) *
          100;
      }
      if (twoQuartersAgoEps !== 0) {
        twoQoQGrowth =
          ((currentQuarterEps - twoQuartersAgoEps) /
            Math.abs(twoQuartersAgoEps)) *
          100;
      }

      return {
        symbol: stock.s,
        name: stock.d?.[columnMap['name']],
        description: stock.d?.[columnMap['description']],
        close: stock.d?.[columnMap['close']],
        change: stock.d?.[columnMap['change']],
        volume: stock.d?.[columnMap['volume']],
        marketCap: stock.d?.[columnMap['market_cap_basic']],
        peRatio: stock.d?.[columnMap['price_earnings_ttm']],
        epsTTM: stock.d?.[columnMap['earnings_per_share_diluted_ttm']],
        epsYoyGrowth:
          stock.d?.[columnMap['earnings_per_share_diluted_yoy_growth_ttm']],
        quarterlyEps: {
          current: currentQuarterEps,
          q1: previousQuarterEps,
          q2: twoQuartersAgoEps,
          q3: threeQuartersAgoEps,
        },
        oneQoQGrowth,
        twoQoQGrowth,
        currentRevenue: stock.d?.[columnMap['total_revenue_fq']],
        previousRevenue: stock.d?.[columnMap['total_revenue_fq1']],
        sector: stock.d?.[columnMap['sector']],
        exchange: stock.d?.[columnMap['exchange']],
        nextEarningsDate: stock.d?.[columnMap['earnings_release_next_date']],
      };
    });

    const validStocks = processedData?.filter(
      (stock: any) =>
        stock.twoQoQGrowth !== null && stock.quarterlyEps.q2 !== 0,
    );

    console.log('Valid stocks:', JSON.stringify(validStocks, null, 2));
    console.log('Total valid stocks:', validStocks?.length || 0);
    console.log('Columns used:', data.columns);
  } catch (error) {
    console.error('TradingView API error:', error);
  }
}

main();
