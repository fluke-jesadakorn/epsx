import WebSocket from 'ws';
import { v4 as uuid } from 'uuid';

const ws = new WebSocket('wss://data.tradingview.com/socket.io/websocket');

ws.on('open', () => {
  // 1. create a new chart session
  const session = 'cs_' + uuid();
  ws.send(JSON.stringify({ m: 'chart_create_session', p: [session, ''] }));

  // 2. resolve the symbol
  ws.send(
    JSON.stringify({
      m: 'resolve_symbol',
      p: [
        session,
        'symbol_1', // any client tag
        'NASDAQ:AAPL', // your symbol
        {
          // optional overrides:
          flags: ['force_permission'],
        },
      ],
    }),
  );

  // when resolve_symbol returns, you'll get an object containing `symbol_session_id`
  // and `symbol_info` including `ticker` and exchange details.

  // 3. request history for a specific date
  //    here we pull daily bars for April 18, 2025 00:00–23:59 UTC
  const from = Math.floor(new Date('2025-04-18T00:00:00Z').getTime() / 1000);
  const to = Math.floor(new Date('2025-04-18T23:59:59Z').getTime() / 1000);

  ws.send(
    JSON.stringify({
      m: 'get_bars',
      p: [
        session,
        'symbol_1', // same client tag you used above
        'D', // resolution = Daily
        from,
        to,
        {
          // optional: you can ask for “adjusted” bars:
          adjust: true,
        },
      ],
    }),
  );
});

ws.on('message', (data) => {
  const msg = JSON.parse(data);
  if (msg.m === 'timescale_update' || msg.m === 'history_update') {
    // msg.p will be an array of bar objects or arrays:
    // e.g. [ { i: 0, v: [timestamp, open, high, low, close, volume] }, … ]
    console.log('Bars:', msg.p);
  }
});
