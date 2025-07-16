/**
 * Example usage of the new EPS vs Price comparison functionality
 * 
 * This demonstrates how to use the new logic that compares:
 * - Last EPS growth (previous quarter)
 * - Current price growth (current quarter)
 */

import type { StockFinancialData } from '@/types/financialChartData';
import {
  getLastEpsVsCurrentPriceComparison,
  formatEpsVsPriceComparison,
  getPriceEpsAlignment,
} from '@/utils/processStocks/stockDataTransformer';

// Example stock data
const exampleStockData: StockFinancialData = {
  symbol: 'AAPL',
  quarters: [
    {
      price: 150.0,
      date: '2024-12-31',
      eps: 1.25,
      quarter: 4,
      eps_growth: undefined, // Current quarter - no growth calculated yet
      price_growth: 8, // Current price grew 8%
      last_eps_vs_current_price: {
        lastEpsGrowth: 15, // Previous quarter EPS grew 15%
        currentPriceGrowth: 8, // Current price grew 8%
      },
    },
    {
      price: 138.9,
      date: '2024-09-30',
      eps: 1.08,
      quarter: 3,
      eps_growth: 15, // Previous quarter EPS grew 15%
      price_growth: 12,
      last_eps_vs_current_price: undefined,
    },
    {
      price: 124.0,
      date: '2024-06-30',
      eps: 0.94,
      quarter: 2,
      eps_growth: 8,
      price_growth: 5,
      last_eps_vs_current_price: undefined,
    },
  ],
  currentPrice: 152.0,
  currentPriceDate: '2025-01-15',
};

// Example usage
function demonstrateEpsPriceComparison() {
  console.log('🔍 EPS vs Price Comparison Example');
  console.log('=====================================');
  
  // Get the comparison data
  const comparison = getLastEpsVsCurrentPriceComparison(exampleStockData);
  console.log('Raw comparison data:', comparison);
  
  // Format for display
  const formatted = formatEpsVsPriceComparison(comparison);
  console.log('Formatted comparison:', formatted);
  
  // Get alignment analysis
  const alignment = getPriceEpsAlignment(comparison);
  console.log('Price-EPS alignment:', alignment);
  
  // Interpretation
  if (comparison) {
    const interpretation = interpretComparison(comparison, alignment);
    console.log('Interpretation:', interpretation);
  }
}

function interpretComparison(
  comparison: { lastEpsGrowth: number | null; currentPriceGrowth: number | null },
  alignment: 'positive' | 'negative' | 'neutral' | null
): string {
  if (!comparison.lastEpsGrowth || !comparison.currentPriceGrowth) {
    return 'Insufficient data for analysis';
  }
  
  const epsGrowth = comparison.lastEpsGrowth;
  const priceGrowth = comparison.currentPriceGrowth;
  
  switch (alignment) {
    case 'positive':
      return `✅ Good alignment: When EPS grew ${epsGrowth}% last quarter, current price is growing ${priceGrowth}% in the same direction.`;
    case 'negative':
      return `❌ Misalignment: Despite EPS growing ${epsGrowth}% last quarter, current price is moving ${priceGrowth}% in the opposite direction.`;
    case 'neutral':
      return `⚖️ Neutral: EPS and price movements are minimal or unclear.`;
    default:
      return 'Unable to determine alignment';
  }
}

// Example scenarios
const scenarios = [
  {
    name: 'Strong Positive Alignment',
    data: { lastEpsGrowth: 20, currentPriceGrowth: 15 },
    description: 'Strong EPS growth followed by strong price growth'
  },
  {
    name: 'Negative Alignment (Divergence)',
    data: { lastEpsGrowth: 18, currentPriceGrowth: -5 },
    description: 'Strong EPS growth but price is declining'
  },
  {
    name: 'Recovery Alignment',
    data: { lastEpsGrowth: -10, currentPriceGrowth: -3 },
    description: 'Both EPS and price declining, but price declining less'
  },
];

function demonstrateScenarios() {
  console.log('\n📊 Different Scenarios:');
  console.log('========================');
  
  scenarios.forEach((scenario, index) => {
    console.log(`\n${index + 1}. ${scenario.name}:`);
    console.log(`   Description: ${scenario.description}`);
    
    const alignment = getPriceEpsAlignment(scenario.data);
    const interpretation = interpretComparison(scenario.data, alignment);
    
    console.log(`   Data: EPS ${scenario.data.lastEpsGrowth}% → Price ${scenario.data.currentPriceGrowth}%`);
    console.log(`   Alignment: ${alignment}`);
    console.log(`   Analysis: ${interpretation}`);
  });
}

// Run examples
demonstrateEpsPriceComparison();
demonstrateScenarios();

export {
  demonstrateEpsPriceComparison,
  demonstrateScenarios,
  interpretComparison,
};
