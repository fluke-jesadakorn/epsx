fetch(
  'https://scanner.tradingview.com/global/scan?label-product=screener-stock',
  {
    headers: {
      accept: 'application/json',
      'accept-language': 'th-TH,th;q=0.9,en;q=0.8',
      'cache-control': 'no-cache',
      'content-type': 'text/plain;charset=UTF-8',
      pragma: 'no-cache',
      priority: 'u=1, i',
      'sec-ch-ua':
        '"Google Chrome";v="137", "Chromium";v="137", "Not/A)Brand";v="24"',
      'sec-ch-ua-mobile': '?0',
      'sec-ch-ua-platform': '"macOS"',
      'sec-fetch-dest': 'empty',
      'sec-fetch-mode': 'cors',
      'sec-fetch-site': 'same-site',
    },
    referrer: 'https://www.tradingview.com/',
    referrerPolicy: 'origin-when-cross-origin',
    body: '{"columns":["name","description","logoid","update_mode","type","typespecs","close","pricescale","minmov","fractional","minmove2","currency","change","volume","relative_volume_10d_calc","market_cap_basic","fundamental_currency_code","price_earnings_ttm","earnings_per_share_diluted_ttm","earnings_per_share_diluted_yoy_growth_ttm","dividends_yield_current","sector.tr","market","sector","recommendation_mark","exchange"],"filter":[{"left":"earnings_per_share_diluted_qoq_growth_fq","operation":"greater","right":0},{"left":"is_primary","operation":"equal","right":true}],"ignore_unknown_fields":false,"options":{"lang":"en"},"price_conversion":{"to_currency":"usd"},"range":[0,100],"sort":{"sortBy":"market_cap_basic","sortOrder":"desc"},"symbols":{},"markets":["america","argentina","australia","austria","bahrain","bangladesh","belgium","brazil","canada","chile","china","colombia","cyprus","czech","denmark","egypt","estonia","finland","france","germany","greece","hongkong","hungary","iceland","india","indonesia","ireland","israel","italy","japan","kenya","kuwait","latvia","lithuania","luxembourg","malaysia","mexico","morocco","netherlands","newzealand","nigeria","norway","pakistan","peru","philippines","poland","portugal","qatar","romania","russia","ksa","serbia","singapore","slovakia","rsa","korea","spain","srilanka","sweden","switzerland","taiwan","thailand","tunisia","turkey","uae","uk","venezuela","vietnam"],"filter2":{"operator":"and","operands":[{"operation":{"operator":"or","operands":[{"operation":{"operator":"and","operands":[{"expression":{"left":"type","operation":"equal","right":"stock"}},{"expression":{"left":"typespecs","operation":"has","right":["common"]}}]}},{"operation":{"operator":"and","operands":[{"expression":{"left":"type","operation":"equal","right":"stock"}},{"expression":{"left":"typespecs","operation":"has","right":["preferred"]}}]}},{"operation":{"operator":"and","operands":[{"expression":{"left":"type","operation":"equal","right":"dr"}}]}},{"operation":{"operator":"and","operands":[{"expression":{"left":"type","operation":"equal","right":"fund"}},{"expression":{"left":"typespecs","operation":"has_none_of","right":["etf"]}}]}}]}}]}}',
    method: 'POST',
    mode: 'cors',
    credentials: 'include',
  },
)
  .then((response) => {
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  })
  .then((data) => {
    console.log('TradingView data:', data);
  });

//  https://scanner.tradingview.com/symbol?symbol=NASDAQ%3ANVDA&fields=sector%2Ccountry%2Cearnings_per_share_basic_fq
