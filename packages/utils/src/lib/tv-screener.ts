fetch(
  'https://scanner.tradingview.com/global/scan?label-product=screener-stock',
  {
    headers: {
      accept: 'application/json',
      'accept-language': 'th-TH,th;q=0.9,en;q=0.8',
      'content-type': 'application/json',
      priority: 'u=1, i',
      'sec-ch-ua':
        '"Chromium";v="134", "Not:A-Brand";v="24", "Google Chrome";v="134"',
      'sec-ch-ua-mobile': '?0',
      'sec-ch-ua-platform': '"macOS"',
      'sec-fetch-dest': 'empty',
      'sec-fetch-mode': 'cors',
      'sec-fetch-site': 'same-site',
      cookie:
        'cookiePrivacyPreferenceBannerProduction=notApplicable; cookiesSettings={"analytics":true,"advertising":true}; _ga=GA1.1.287998567.1737519655; device_t=Yk1Fb0FROjI.o614l3RjSdzzD_nBlfv7bHmRO9SLuSg8Jdc5C8_Cfyg; sessionid=8muioxxaoyz3d8lvyqey7wa6fnmhzx4f; sessionid_sign=v3:snfqsmVt+TvwXS43RnT/4MEx8OEh36wZ7gWDGEFK+BY=; tv_ecuid=e538a49d-5181-4179-b95f-35fc057358a6; _sp_ses.cf1a=*; __gads=ID=a2dce82dea5f6034:T=1741599088:RT=1741916739:S=ALNI_MZVTo4XKZH5jxKH9m9ofmqw5e75WQ; __gpi=UID=0000105bb6af1052:T=1741599088:RT=1741916739:S=ALNI_MZae1ihNv6rQ0C6ewUUoX6UxQjcgg; __eoi=ID=c798787be475c469:T=1741599088:RT=1741916739:S=AA-AfjbGRWxAkTkVM-VaX35P2o4H; _ga_YVVRYGL0E0=GS1.1.1741916372.121.1.1741916739.60.0.0; _sp_id.cf1a=b0388acf-1e28-423e-9d8e-0d5a55fad2e9.1737519655.86.1741916853.1741876748.4f22c022-683f-436d-ad7d-e4664f725e5f.2953e049-e782-404c-a43f-5fe2f14055b5.9173542d-90b8-4a22-a769-f2c656896ba6.1741914535100.62',
      Referer: 'https://www.tradingview.com/',
      'Referrer-Policy': 'origin-when-cross-origin',
    },
    body: JSON.stringify({
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
        'earnings_per_share_diluted_fq',
        'fundamental_currency_code',
        'earnings_per_share_diluted_qoq_growth_fq',
        'volume',
        'market_cap_basic',
        'price_earnings_ttm',
        'sector.tr',
        'market',
        'sector',
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
      range: [0, 200],
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
              operator: 'and',
              operands: [
                {
                  expression: {
                    left: 'recommendation_mark',
                    operation: 'nequal',
                    right: 1.25,
                  },
                },
              ],
            },
          },
          {
            operation: {
              operator: 'or',
              operands: [
                {
                  expression: {
                    left: 'recommendation_mark',
                    operation: 'in_range',
                    right: [1, 1.25],
                  },
                },
              ],
            },
          },
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
    }),
    method: 'POST',
  },
)
  .then((response) => response.json())
  .then((data) => console.log(data));
  fetch(
    'https://scanner.tradingview.com/global/scan?label-product=underchart-screener-stock',
    {
      headers: {
        accept: 'text/plain, */*; q=0.01',
        'accept-language': 'th-TH,th;q=0.9,en;q=0.8',
        'content-type': 'application/json',
        priority: 'u=1, i',
        'sec-ch-ua':
          '"Chromium";v="134", "Not:A-Brand";v="24", "Google Chrome";v="134"',
        'sec-ch-ua-mobile': '?0',
        'sec-ch-ua-platform': '"macOS"',
        'sec-fetch-dest': 'empty',
        'sec-fetch-mode': 'cors',
        'sec-fetch-site': 'same-site',
        cookie:
          'cookiePrivacyPreferenceBannerProduction=notApplicable; cookiesSettings={"analytics":true,"advertising":true}; _ga=GA1.1.287998567.1737519655; device_t=Yk1Fb0FROjI.o614l3RjSdzzD_nBlfv7bHmRO9SLuSg8Jdc5C8_Cfyg; sessionid=8muioxxaoyz3d8lvyqey7wa6fnmhzx4f; sessionid_sign=v3:snfqsmVt+TvwXS43RnT/4MEx8OEh36wZ7gWDGEFK+BY=; tv_ecuid=e538a49d-5181-4179-b95f-35fc057358a6; _sp_ses.cf1a=*; __gads=ID=a2dce82dea5f6034:T=1741599088:RT=1741922022:S=ALNI_MZVTo4XKZH5jxKH9m9ofmqw5e75WQ; __gpi=UID=0000105bb6af1052:T=1741599088:RT=1741922022:S=ALNI_MZae1ihNv6rQ0C6ewUUoX6UxQjcgg; __eoi=ID=c798787be475c469:T=1741599088:RT=1741922022:S=AA-AfjbGRWxAkTkVM-VaX35P2o4H; _sp_id.cf1a=b0388acf-1e28-423e-9d8e-0d5a55fad2e9.1737519655.86.1741922337.1741876748.4f22c022-683f-436d-ad7d-e4664f725e5f.2953e049-e782-404c-a43f-5fe2f14055b5.9173542d-90b8-4a22-a769-f2c656896ba6.1741914535100.179; _ga_YVVRYGL0E0=GS1.1.1741916372.121.1.1741922390.60.0.0',
        Referer: 'https://www.tradingview.com/',
        'Referrer-Policy': 'origin-when-cross-origin',
      },
      body: JSON.stringify({
        filter: [
          { left: 'is_primary', operation: 'equal', right: true },
          { left: 'last_annual_eps', operation: 'greater', right: 0 },
          { left: 'earnings_per_share_forecast_next_fq', operation: 'greater', right: 0 },
          { left: 'earnings_per_share_diluted_qoq_growth_fq', operation: 'greater', right: 0 }
        ],
        options: { lang: 'en' },
        markets: ['america','uk','india','spain','russia','australia','brazil','japan','newzealand','turkey','switzerland','hongkong','taiwan','netherlands','belgium','portugal','france','mexico','canada','colombia','uae','nigeria','singapore','germany','pakistan','peru','poland','italy','argentina','israel','ireland','egypt','srilanka','serbia','chile','china','malaysia','morocco','ksa','bahrain','qatar','indonesia','finland','iceland','denmark','romania','hungary','sweden','slovakia','lithuania','luxembourg','estonia','latvia','vietnam','rsa','thailand','tunisia','korea','kenya','kuwait','norway','philippines','greece','venezuela','cyprus','bangladesh','austria','czech'],
        symbols: {
          query: { types: [] },
          tickers: []
        },
        columns: ['logoid','name','close','change','change_abs','Recommend.All','volume','Value.Traded','market_cap_basic','price_earnings_ttm','earnings_per_share_basic_ttm','number_of_employees','sector','description','type','subtype','update_mode','pricescale','minmov','fractional','minmove2','currency','fundamental_currency_code'],
        sort: {
          sortBy: 'volume',
          sortOrder: 'desc'
        },
        range: [0, 150]
      }),
      method: 'POST',
    },
  );
