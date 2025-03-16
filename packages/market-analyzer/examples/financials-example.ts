import { getFinancials } from '../src';

async function main() {
  try {
    // Get financial data for a stock
    const aaplFinancials = await getFinancials('NASDAQ:AAPL');
    console.log('AAPL Financials:', aaplFinancials);

  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  }
}

// Run the example
main().catch(console.error);

/* Example output:
AAPL Financials: {
  symbol: "NASDAQ:AAPL",
  data: {
    description: "Apple Inc.",
    exchange: "NASDAQ",
    type: "stock",
    currency: "USD",
    fundamental: {
      // Financial metrics and ratios
    }
  },
  status: "success"
}
*/
