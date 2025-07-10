const { getFinancialsFromChart } = require('./getPriceAndEps');

// Test with a small sample of symbols to verify the fix
const testSymbols = ['AAPL', 'MSFT', 'GOOGL'];

async function testPriceFix() {
  console.log('Testing price fix...');
  console.log('Symbols to test:', testSymbols);
  
  try {
    const results = await getFinancialsFromChart(testSymbols, 3);
    
    console.log('\n=== RESULTS ===');
    Object.entries(results).forEach(([symbol, data]) => {
      console.log(`\n${symbol}:`);
      data.forEach((item, index) => {
        console.log(`  Quarter ${index + 1}: Price=${item.price}, Date=${item.date}, EPS=${item.eps}`);
      });
      
      // Check if prices are identical (the bug we're fixing)
      const prices = data.map(item => item.price).filter(p => p !== null);
      const uniquePrices = [...new Set(prices)];
      
      if (prices.length > 1 && uniquePrices.length === 1) {
        console.log(`  ⚠️  WARNING: All prices are identical (${prices[0]}) - bug may still exist`);
      } else if (uniquePrices.length > 1) {
        console.log(`  ✅ SUCCESS: Found ${uniquePrices.length} different prices`);
      } else if (prices.length === 0) {
        console.log(`  ⚠️  WARNING: No prices found - may need more historical data`);
      }
    });
    
  } catch (error) {
    console.error('Test failed:', error);
  }
}

testPriceFix();
