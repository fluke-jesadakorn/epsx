import fetchScreenerStock from './fetchRankScreenedStock';
import { MarketCountry } from '../../../../types/marketCountries';
import { getFinancialsFromChart } from '../getFinancialsFromChart/getPriceAndEps';

/**
 * Fetches top ranked stocks and returns their price and EPS data from chart.
 * @param {number} skip - Number of stocks to skip (default: 0).
 * @param {number} limit - Maximum number of stocks to fetch (default: 10).
 * @param {string|string[]} country - Market country or countries to filter stocks (default: all markets).
 * @param {number} quarters - Number of quarters to fetch (default: 2).
 * @returns {Promise<Record<string, FinancialsFromChart[]>>} - Mapping of stock symbols to their financials.
 */
export async function rankStocksByEpsWithChart(
  skip = 0,
  limit = 10,
  country: typeof MarketCountry = MarketCountry,
  quarters = 3,
) {
  try {
    // 1. Fetch top ranked stocks
    const stockData = await fetchScreenerStock(skip, limit, country);

    if (!stockData || !stockData.data || stockData.data.length === 0) {
      console.log('No stock data retrieved.');
      return {};
    }

    // 2. Extract symbols in correct format
    const symbols = stockData.data.map((stock: any) => stock.s);

    // 3. Fetch financials from chart for these symbols
    const financials = await getFinancialsFromChart(symbols, quarters);

    // 4. Return result in required format
    return financials;
  } catch (error) {
    console.error('Error in rankStocksByEpsWithChart:', error);
    return {};
  }
}

// Example usage:
rankStocksByEpsWithChart(0, 10, MarketCountry)
  .then((data) => {
    console.log('Financials from chart:', data);
  })
  .catch((error) => {
    console.error('Error fetching financials from chart:', error);
  });
