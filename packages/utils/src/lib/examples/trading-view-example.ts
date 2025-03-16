import TradingViewWebSocket from '../tv-websocket';

const example = async () => {
  const ws = new TradingViewWebSocket();

  // Handle process termination gracefully
  const cleanup = () => {
    console.log('\nCleaning up...');
    ws.destroy();
    process.exit(0);
  };

  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);

  try {
    console.log('Connecting to TradingView WebSocket...');
    await ws.waitForReady();
    console.log('WebSocket connected and ready');

    // Example symbols from different exchanges
    const symbols = [
      {
        symbol: 'BINANCE:BTCUSDT',  // Binance Bitcoin/USDT
        interval: '1D'
      },
      {
        symbol: 'NYSE:TSLA',        // NYSE Tesla
        interval: '1D'
      },
      {
        symbol: 'SET_DLY:PTT',      // Thai Stock PTT
        interval: '1D'
      }
    ];

    // Helper to format price with appropriate decimals
    const formatPrice = (price: number, symbol: string) => {
      if (symbol.includes('USDT')) return price.toFixed(2);
      return price.toFixed(symbol.startsWith('SET_DLY') ? 2 : 2);
    };

    // Subscribe to each symbol
    for (const { symbol, interval } of symbols) {
      console.log(`Subscribing to ${symbol}...`);
      
      try {
        await ws.subscribe({
          symbol,
          interval,
          onUpdate: (data) => {
            const timestamp = new Date(data.timestamp * 1000).toLocaleString();
            console.log(
              `[${timestamp}] ${data.symbol}:\n` +
              `  Price: ${formatPrice(data.price, symbol)}\n` +
              `  Volume: ${data.volume.toLocaleString()}\n` +
              '---'
            );
          }
        });
      } catch (error) {
        console.error(`Failed to subscribe to ${symbol}:`, error);
        // Continue with other symbols even if one fails
        continue;
      }
    }

    console.log('\nListening for price updates (Press Ctrl+C to exit)...\n');

    // Keep the process running
    await new Promise(() => {});

  } catch (error) {
    console.error('Fatal error:', error);
    cleanup();
  }
};

// Run the example if this file is executed directly
if (require.main === module) {
  example().catch(console.error);
}

export default example;
