import { getPrice } from '../src';

async function main() {
  try {
    // Get current price data for multiple stocks
    const aaplPrice = await getPrice('NASDAQ:AAPL');
    console.log('AAPL Current Price:', aaplPrice);

    const tslaPrice = await getPrice('NASDAQ:TSLA');
    console.log('TSLA Current Price:', tslaPrice);

  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  }
}

// Run the example
main().catch(console.error);

/* Example output:
AAPL Current Price: {
  symbol: "NASDAQ:AAPL",
  price: 173.25,
  volume: 1250000,
  timestamp: 1710461577,
  status: "success"
}
TSLA Current Price: {
  symbol: "NASDAQ:TSLA",
  price: 245.10,
  volume: 850000,
  timestamp: 1710461577,
  status: "success"
}
*/
