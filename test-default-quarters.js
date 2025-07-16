// Quick test to verify that the default now returns only 2 quarters (current and previous)
// This should show that:
// 1. Default quarters = 2 
// 2. Returns current quarter and previous quarter
// 3. Still calculates growth properly

import { getFinancialsWithCurrentPriceFromChart } from './apps/frontend/utils/processStocks/getPriceAndEps.ts';

async function testDefaultQuarters() {
  console.log('🧪 Testing default quarters (should be 2)...');
  
  const testSymbol = ['AAPL'];
  
  // Call without specifying quarters - should default to 2
  console.log(`📊 Calling getFinancialsWithCurrentPriceFromChart without quarters parameter`);
  
  try {
    const result = await getFinancialsWithCurrentPriceFromChart(testSymbol);
    
    if (result[testSymbol[0]]) {
      const data = result[testSymbol[0]];
      console.log(`✅ Received ${data.quarters.length} quarters (expected: 2)`);
      
      if (data.quarters.length === 2) {
        console.log('🎉 SUCCESS: Default now returns 2 quarters!');
        console.log('\n📋 Quarters returned (current and previous):');
        
        data.quarters.forEach((q, i) => {
          const label = i === 0 ? 'Previous' : 'Current';
          console.log(`  ${label}: ${q.date} - EPS: ${q.eps}, EPS Growth: ${q.eps_growth}%`);
        });
      } else {
        console.log(`❌ FAILED: Expected 2 quarters but got ${data.quarters.length}`);
      }
    } else {
      console.log('❌ No data received for test symbol');
    }
  } catch (error) {
    console.error('❌ Test failed with error:', error);
  }
}

// Run the test
testDefaultQuarters().catch(console.error);
