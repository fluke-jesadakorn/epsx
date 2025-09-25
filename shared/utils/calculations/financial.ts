/**
 * FINANCIAL CALCULATIONS UTILITIES
 * EPS calculations, financial metrics, and trading calculations
 */

/**
 * Calculate EPS growth rate
 */
export function calculateEPSGrowth(current: number, previous: number): number {
  if (previous === 0) return 0
  return ((current - previous) / Math.abs(previous)) * 100
}

/**
 * Calculate EPS surprise percentage
 */
export function calculateEPSSurprise(actual: number, estimated: number): number {
  if (estimated === 0) return 0
  return ((actual - estimated) / Math.abs(estimated)) * 100
}

/**
 * Calculate estimated growth percentage between two EPS values
 */
export function calculateEstimatedGrowth(estimatedEPS: number, currentEPS: number): number {
  if (!currentEPS || currentEPS === 0) return 0
  return ((estimatedEPS - currentEPS) / currentEPS) * 100
}

/**
 * Calculate P/E ratio from price and EPS
 */
export function calculatePERatio(price: number, eps: number): number {
  if (!eps || eps === 0) return 0
  return price / eps
}

/**
 * Format P/E ratio with proper precision and 'x' suffix
 */
export function formatPERatio(peRatio: number): string {
  if (peRatio === 0 || !isFinite(peRatio)) return 'N/A'
  return `${peRatio.toFixed(1)}x`
}

/**
 * Calculate price growth percentage between two price values
 */
export function calculatePriceGrowth(currentPrice: number, previousPrice: number): number {
  if (!previousPrice || previousPrice === 0) return 0
  return ((currentPrice - previousPrice) / previousPrice) * 100
}

/**
 * Calculate estimated price target based on estimated EPS and current P/E ratio
 */
export function calculatePriceTarget(estimatedEPS: number, currentPE: number): number {
  if (!estimatedEPS || !currentPE || currentPE === 0) return 0
  return estimatedEPS * currentPE
}

/**
 * Calculate percentage change between two values
 */
export function calculatePercentageChange(oldValue: number, newValue: number): number {
  if (oldValue === 0) return newValue === 0 ? 0 : 100
  return ((newValue - oldValue) / oldValue) * 100
}

/**
 * Calculate compound annual growth rate (CAGR)
 */
export function calculateCAGR(beginningValue: number, endingValue: number, periods: number): number {
  if (beginningValue <= 0 || endingValue <= 0 || periods <= 0) return 0
  return (Math.pow(endingValue / beginningValue, 1 / periods) - 1) * 100
}

/**
 * Calculate return on equity (ROE)
 */
export function calculateROE(netIncome: number, shareholderEquity: number): number {
  if (shareholderEquity === 0) return 0
  return (netIncome / shareholderEquity) * 100
}

/**
 * Calculate return on assets (ROA)
 */
export function calculateROA(netIncome: number, totalAssets: number): number {
  if (totalAssets === 0) return 0
  return (netIncome / totalAssets) * 100
}

/**
 * Calculate debt-to-equity ratio
 */
export function calculateDebtToEquity(totalDebt: number, totalEquity: number): number {
  if (totalEquity === 0) return 0
  return totalDebt / totalEquity
}

/**
 * Calculate current ratio
 */
export function calculateCurrentRatio(currentAssets: number, currentLiabilities: number): number {
  if (currentLiabilities === 0) return 0
  return currentAssets / currentLiabilities
}

/**
 * Calculate quick ratio (acid-test ratio)
 */
export function calculateQuickRatio(
  currentAssets: number, 
  inventory: number, 
  currentLiabilities: number
): number {
  if (currentLiabilities === 0) return 0
  return (currentAssets - inventory) / currentLiabilities
}

/**
 * Calculate gross profit margin
 */
export function calculateGrossMargin(revenue: number, costOfGoodsSold: number): number {
  if (revenue === 0) return 0
  return ((revenue - costOfGoodsSold) / revenue) * 100
}

/**
 * Calculate operating margin
 */
export function calculateOperatingMargin(operatingIncome: number, revenue: number): number {
  if (revenue === 0) return 0
  return (operatingIncome / revenue) * 100
}

/**
 * Calculate net profit margin
 */
export function calculateNetMargin(netIncome: number, revenue: number): number {
  if (revenue === 0) return 0
  return (netIncome / revenue) * 100
}

/**
 * Calculate book value per share
 */
export function calculateBookValuePerShare(totalEquity: number, sharesOutstanding: number): number {
  if (sharesOutstanding === 0) return 0
  return totalEquity / sharesOutstanding
}

/**
 * Calculate price-to-book ratio
 */
export function calculatePriceToBook(stockPrice: number, bookValuePerShare: number): number {
  if (bookValuePerShare === 0) return 0
  return stockPrice / bookValuePerShare
}

/**
 * Calculate dividend yield
 */
export function calculateDividendYield(annualDividend: number, stockPrice: number): number {
  if (stockPrice === 0) return 0
  return (annualDividend / stockPrice) * 100
}

/**
 * Calculate dividend payout ratio
 */
export function calculatePayoutRatio(dividendPerShare: number, earningsPerShare: number): number {
  if (earningsPerShare === 0) return 0
  return (dividendPerShare / earningsPerShare) * 100
}

/**
 * Calculate enterprise value
 */
export function calculateEnterpriseValue(
  marketCap: number,
  totalDebt: number,
  cashAndEquivalents: number
): number {
  return marketCap + totalDebt - cashAndEquivalents
}

/**
 * Calculate EV/EBITDA ratio
 */
export function calculateEVToEBITDA(enterpriseValue: number, ebitda: number): number {
  if (ebitda === 0) return 0
  return enterpriseValue / ebitda
}

/**
 * Calculate asset turnover ratio
 */
export function calculateAssetTurnover(revenue: number, averageTotalAssets: number): number {
  if (averageTotalAssets === 0) return 0
  return revenue / averageTotalAssets
}

/**
 * Calculate inventory turnover ratio
 */
export function calculateInventoryTurnover(costOfGoodsSold: number, averageInventory: number): number {
  if (averageInventory === 0) return 0
  return costOfGoodsSold / averageInventory
}

/**
 * Calculate receivables turnover ratio
 */
export function calculateReceivablesTurnover(revenue: number, averageAccountsReceivable: number): number {
  if (averageAccountsReceivable === 0) return 0
  return revenue / averageAccountsReceivable
}

/**
 * Calculate working capital
 */
export function calculateWorkingCapital(currentAssets: number, currentLiabilities: number): number {
  return currentAssets - currentLiabilities
}

/**
 * Calculate market capitalization
 */
export function calculateMarketCap(stockPrice: number, sharesOutstanding: number): number {
  return stockPrice * sharesOutstanding
}

/**
 * Calculate earnings yield (inverse of P/E ratio)
 */
export function calculateEarningsYield(earningsPerShare: number, stockPrice: number): number {
  if (stockPrice === 0) return 0
  return (earningsPerShare / stockPrice) * 100
}

/**
 * Calculate free cash flow
 */
export function calculateFreeCashFlow(operatingCashFlow: number, capitalExpenditures: number): number {
  return operatingCashFlow - capitalExpenditures
}

/**
 * Calculate free cash flow yield
 */
export function calculateFCFYield(freeCashFlow: number, marketCap: number): number {
  if (marketCap === 0) return 0
  return (freeCashFlow / marketCap) * 100
}

/**
 * Calculate beta (systematic risk)
 */
export function calculateBeta(stockReturns: number[], marketReturns: number[]): number {
  if (stockReturns.length !== marketReturns.length || stockReturns.length === 0) return 0
  
  const stockMean = stockReturns.reduce((sum, val) => sum + val, 0) / stockReturns.length
  const marketMean = marketReturns.reduce((sum, val) => sum + val, 0) / marketReturns.length
  
  let covariance = 0
  let marketVariance = 0
  
  for (let i = 0; i < stockReturns.length; i++) {
    const stockDev = stockReturns[i] - stockMean
    const marketDev = marketReturns[i] - marketMean
    covariance += stockDev * marketDev
    marketVariance += marketDev * marketDev
  }
  
  if (marketVariance === 0) return 0
  return covariance / marketVariance
}

/**
 * Calculate Sharpe ratio
 */
export function calculateSharpeRatio(
  portfolioReturn: number,
  riskFreeRate: number,
  portfolioStandardDeviation: number
): number {
  if (portfolioStandardDeviation === 0) return 0
  return (portfolioReturn - riskFreeRate) / portfolioStandardDeviation
}