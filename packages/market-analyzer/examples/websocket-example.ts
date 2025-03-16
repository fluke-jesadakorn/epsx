import { PriceWebSocket, FinancialsWebSocket } from '../src';

async function main() {
  try {
    // Initialize WebSocket instances
    const priceWs = new PriceWebSocket();
    const financialsWs = new FinancialsWebSocket();

    // Initialize WebSocket connections
    await priceWs.waitForReady();
    await financialsWs.waitForReady();
    
    // Subscribe to real-time price updates
    await priceWs.subscribe({
      symbol: 'NASDAQ:AAPL',
      onUpdate: (update) => {
        console.log('AAPL Update:', {
          price: update.price,
          volume: update.volume,
          timestamp: new Date(update.timestamp * 1000).toISOString()
        });
      }
    });

    await priceWs.subscribe({
      symbol: 'NASDAQ:TSLA',
      onUpdate: (update) => {
        console.log('TSLA Update:', {
          price: update.price,
          volume: update.volume,
          timestamp: new Date(update.timestamp * 1000).toISOString()
        });
      }
    });

    // Get real-time financial data
    const realtimeFinancials = await financialsWs.getFinancials('NASDAQ:AAPL');
    console.log('AAPL Real-time Financials:', realtimeFinancials);

    // Keep WebSockets running for continuous updates
    setTimeout(() => {
      console.log('Disconnecting websockets...');
      priceWs.disconnect();
      financialsWs.disconnect();
    }, 60000);

  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  }
}

// Run the example
main().catch(console.error);

/* Example output:
AAPL Update: {
  price: 173.25,
  volume: 1250000,
  timestamp: "2024-03-15T05:42:57.000Z"
}
TSLA Update: {
  price: 245.10,
  volume: 850000,
  timestamp: "2024-03-15T05:42:57.000Z"
}
*/
