// Test script to verify the EPS quarters fix
// This tests that when requesting 2 quarters, the system:
// 1. Fetches enough data (4+ quarters) for proper calculations
// 2. Returns only 2 quarters to the frontend
// 3. Ensures growth calculations work properly

import { getFinancialsWithCurrentPriceFromChart } from './apps/frontend/utils/processStocks/getPriceAndEps.ts';

async function testEpsFix() {
  console.log('🧪 Testing EPS quarters fix...');
  
  // Test with a sample symbol requesting only 2 quarters
  const testSymbol = ['AAPL'];
  const requestedQuarters = 2;
  
  console.log(`📊 Requesting ${requestedQuarters} quarters for ${testSymbol[0]}`);
  
  try {
    const result = await getFinancialsWithCurrentPriceFromChart(testSymbol, requestedQuarters);
    
    if (result[testSymbol[0]]) {
      const data = result[testSymbol[0]];
      console.log(`✅ Received ${data.quarters.length} quarters (expected: ${requestedQuarters})`);
      
      // Check if we have proper growth calculations
      const hasEpsGrowth = data.quarters.some(q => q.eps_growth !== undefined && q.eps_growth !== null);
      const hasPriceGrowth = data.quarters.some(q => q.price_growth !== undefined && q.price_growth !== null);
      const hasComparison = data.quarters.some(q => q.last_eps_vs_current_price !== undefined);
      
      console.log(`📈 EPS Growth calculations: ${hasEpsGrowth ? '✅ Present' : '❌ Missing'}`);
      console.log(`💰 Price Growth calculations: ${hasPriceGrowth ? '✅ Present' : '❌ Missing'}`);
      console.log(`🔍 EPS vs Price comparison: ${hasComparison ? '✅ Present' : '❌ Missing'}`);
      
      // Log the quarters data
      console.log('\n📋 Quarters data:');
      data.quarters.forEach((q, i) => {
        console.log(`  Q${i + 1}: ${q.date} - EPS: ${q.eps}, EPS Growth: ${q.eps_growth}, Price Growth: ${q.price_growth}`);
        if (q.last_eps_vs_current_price) {
          console.log(`      Comparison: Last EPS Growth: ${q.last_eps_vs_current_price.lastEpsGrowth}, Current Price Growth: ${q.last_eps_vs_current_price.currentPriceGrowth}`);
        }
      });
      
      if (data.quarters.length === requestedQuarters && hasEpsGrowth && hasComparison) {
        console.log('\n🎉 Test PASSED: Fix working correctly!');
      } else {
        console.log('\n❌ Test FAILED: Issues detected');
      }
    } else {
      console.log('❌ No data received for test symbol');
    }
  } catch (error) {
    console.error('❌ Test failed with error:', error);
  }
}

// Run the test
testEpsFix().catch(console.error);
