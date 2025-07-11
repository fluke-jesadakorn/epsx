import { MarketCountry } from '../../../../types/marketCountries';

const fetchScreenerStock = async (
  skip = 0,
  limit = 5,
  country = MarketCountry,
) => {
  const customBody = () =>
    JSON.stringify({
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
        'market_cap_basic',
        'fundamental_currency_code',
        'price_earnings_ttm',
        'earnings_per_share_diluted_ttm',
        'sector.tr',
        'market',
        'sector',
        'recommendation_mark',
        'earnings_per_share_diluted_qoq_growth_fq',
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
      range: [skip, limit],
      sort: {
        sortBy: 'market_cap_basic',
        sortOrder: 'desc',
      },
      symbols: {},
      markets: typeof country === 'string' ? [country] : Object.values(country),
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

  const response = await fetch(
    'https://scanner.tradingview.com/global/scan?label-product=screener-stock',
    {
      headers: {
        accept: 'application/json',
        'accept-language': 'th-TH,th;q=0.9,en;q=0.8',
        'cache-control': 'no-cache',
        'content-type': 'text/plain;charset=UTF-8',
        pragma: 'no-cache',
        priority: 'u=1, i',
        'sec-ch-ua': '"Not)A;Brand";v="8", "Chromium";v="138", "Google Chrome";v="138"',
        'sec-ch-ua-mobile': '?0',
        'sec-ch-ua-platform': '"macOS"',
        'sec-fetch-dest': 'empty',
        'sec-fetch-mode': 'cors',
        'sec-fetch-site': 'same-site',
        cookie: 'cookiePrivacyPreferenceBannerProduction=notApplicable; cookiesSettings={"analytics":true,"advertising":true}; _ga=GA1.1.287998567.1737519655; device_t=Yk1Fb0FROjI.o614l3RjSdzzD_nBlfv7bHmRO9SLuSg8Jdc5C8_Cfyg; sessionid=8muioxxaoyz3d8lvyqey7wa6fnmhzx4f; sessionid_sign=v3:snfqsmVt+TvwXS43RnT/4MEx8OEh36wZ7gWDGEFK+BY=; tv_ecuid=e538a49d-5181-4179-b95f-35fc057358a6; __gads=ID=a2dce82dea5f6034:T=1741599088:RT=1746415506:S=ALNI_MZVTo4XKZH5jxKH9m9ofmqw5e75WQ; __gpi=UID=0000105bb6af1052:T=1741599088:RT=1746415506:S=ALNI_MZae1ihNv6rQ0C6ewUUoX6UxQjcgg; __eoi=ID=c798787be475c469:T=1741599088:RT=1746415506:S=AA-AfjbGRWxAkTkVM-VaX35P2o4H; _sp_ses.cf1a=*; _ga_YVVRYGL0E0=GS2.1.s1752193235$o314$g1$t1752196268$j43$l0$h0; _sp_id.cf1a=b0388acf-1e28-423e-9d8e-0d5a55fad2e9.1737519655.197.1752196303.1752191132.afe82221-1b0e-4654-8e22-e8aa7494e904.1ad57957-27a4-4b7c-89ee-8e996803b5cf.8a1335d9-728f-405e-b586-5c71cf473d5f.1752193235131.46',
        Referer: 'https://www.tradingview.com/',
      },
      body: customBody(),
      method: 'POST',
    },
  );
  if (!response.ok) {
    throw new Error(
      `Failed to fetch screener stock data: ${response.statusText}`,
    );
  }
  const data = await response.json();
  return data;
};
export default fetchScreenerStock;
