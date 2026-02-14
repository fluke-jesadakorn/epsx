import type { MockHandler } from '../types';

const stocks = [
  { id: 1, symbol: 'AAPL', name: 'Apple Inc.', price: 189.84, change: 2.15, change_pct: 1.15, volume: 52438900, market_cap: 2950000000000, sector: 'Technology', country: 'US', exchange: 'NASDAQ', rank: 1, score: 98.5 },
  { id: 2, symbol: 'TSLA', name: 'Tesla Inc.', price: 248.42, change: -3.78, change_pct: -1.50, volume: 98234100, market_cap: 789000000000, sector: 'Consumer Cyclical', country: 'US', exchange: 'NASDAQ', rank: 2, score: 95.2 },
  { id: 3, symbol: 'MSFT', name: 'Microsoft Corp.', price: 415.60, change: 5.32, change_pct: 1.30, volume: 21345600, market_cap: 3100000000000, sector: 'Technology', country: 'US', exchange: 'NASDAQ', rank: 3, score: 94.8 },
  { id: 4, symbol: 'NVDA', name: 'NVIDIA Corp.', price: 878.35, change: 12.40, change_pct: 1.43, volume: 45678900, market_cap: 2170000000000, sector: 'Technology', country: 'US', exchange: 'NASDAQ', rank: 4, score: 93.1 },
  { id: 5, symbol: 'AMZN', name: 'Amazon.com Inc.', price: 178.25, change: 1.85, change_pct: 1.05, volume: 34567800, market_cap: 1860000000000, sector: 'Consumer Cyclical', country: 'US', exchange: 'NASDAQ', rank: 5, score: 91.7 },
  { id: 6, symbol: 'GOOGL', name: 'Alphabet Inc.', price: 141.80, change: -0.92, change_pct: -0.64, volume: 23456700, market_cap: 1780000000000, sector: 'Technology', country: 'US', exchange: 'NASDAQ', rank: 6, score: 90.3 },
  { id: 7, symbol: 'META', name: 'Meta Platforms Inc.', price: 485.20, change: 8.15, change_pct: 1.71, volume: 18234500, market_cap: 1240000000000, sector: 'Technology', country: 'US', exchange: 'NASDAQ', rank: 7, score: 89.6 },
  { id: 8, symbol: 'BRK.B', name: 'Berkshire Hathaway', price: 412.30, change: 0.45, change_pct: 0.11, volume: 3456700, market_cap: 897000000000, sector: 'Financial Services', country: 'US', exchange: 'NYSE', rank: 8, score: 88.2 },
  { id: 9, symbol: 'TSM', name: 'Taiwan Semiconductor', price: 142.75, change: 3.20, change_pct: 2.29, volume: 15678900, market_cap: 740000000000, sector: 'Technology', country: 'TW', exchange: 'NYSE', rank: 9, score: 87.5 },
  { id: 10, symbol: 'SAP', name: 'SAP SE', price: 195.40, change: 1.10, change_pct: 0.57, volume: 2345600, market_cap: 238000000000, sector: 'Technology', country: 'DE', exchange: 'NYSE', rank: 10, score: 86.1 },
  { id: 11, symbol: 'JNJ', name: 'Johnson & Johnson', price: 156.80, change: -1.25, change_pct: -0.79, volume: 7890100, market_cap: 378000000000, sector: 'Healthcare', country: 'US', exchange: 'NYSE', rank: 11, score: 85.4 },
  { id: 12, symbol: 'V', name: 'Visa Inc.', price: 278.90, change: 2.45, change_pct: 0.89, volume: 6543200, market_cap: 572000000000, sector: 'Financial Services', country: 'US', exchange: 'NYSE', rank: 12, score: 84.8 },
  { id: 13, symbol: 'UNH', name: 'UnitedHealth Group', price: 527.15, change: -4.30, change_pct: -0.81, volume: 3456700, market_cap: 489000000000, sector: 'Healthcare', country: 'US', exchange: 'NYSE', rank: 13, score: 83.2 },
  { id: 14, symbol: 'SHEL', name: 'Shell plc', price: 68.45, change: 0.78, change_pct: 1.15, volume: 4567800, market_cap: 214000000000, sector: 'Energy', country: 'GB', exchange: 'NYSE', rank: 14, score: 82.7 },
  { id: 15, symbol: 'NVO', name: 'Novo Nordisk A/S', price: 125.30, change: 6.20, change_pct: 5.21, volume: 8901200, market_cap: 560000000000, sector: 'Healthcare', country: 'DK', exchange: 'NYSE', rank: 15, score: 81.9 },
  { id: 16, symbol: 'WMT', name: 'Walmart Inc.', price: 172.55, change: 0.95, change_pct: 0.55, volume: 5678900, market_cap: 465000000000, sector: 'Consumer Defensive', country: 'US', exchange: 'NYSE', rank: 16, score: 80.4 },
  { id: 17, symbol: 'JPM', name: 'JPMorgan Chase', price: 198.40, change: 3.10, change_pct: 1.59, volume: 8765400, market_cap: 571000000000, sector: 'Financial Services', country: 'US', exchange: 'NYSE', rank: 17, score: 79.8 },
  { id: 18, symbol: 'TM', name: 'Toyota Motor Corp.', price: 232.10, change: -1.80, change_pct: -0.77, volume: 1234500, market_cap: 320000000000, sector: 'Consumer Cyclical', country: 'JP', exchange: 'NYSE', rank: 18, score: 78.3 },
  { id: 19, symbol: 'ASML', name: 'ASML Holding N.V.', price: 985.60, change: 15.40, change_pct: 1.59, volume: 2345600, market_cap: 398000000000, sector: 'Technology', country: 'NL', exchange: 'NASDAQ', rank: 19, score: 77.6 },
  { id: 20, symbol: 'PG', name: 'Procter & Gamble', price: 162.35, change: 0.60, change_pct: 0.37, volume: 4567800, market_cap: 382000000000, sector: 'Consumer Defensive', country: 'US', exchange: 'NYSE', rank: 20, score: 76.1 },
];

const filters = {
  countries: ['US', 'TW', 'DE', 'GB', 'DK', 'JP', 'NL', 'KR', 'IN', 'BR'],
  sectors: ['Technology', 'Healthcare', 'Financial Services', 'Consumer Cyclical', 'Consumer Defensive', 'Energy', 'Industrials'],
  exchanges: ['NASDAQ', 'NYSE', 'LSE', 'TSE', 'XETRA'],
  sort_options: ['rank', 'price', 'change_pct', 'volume', 'market_cap', 'score'],
};

export const analyticsMocks: MockHandler[] = [
  {
    pattern: '**/api/analytics/rankings**',
    handler: (url: URL) => {
      const page = parseInt(url.searchParams.get('page') ?? '1');
      const search = url.searchParams.get('search') ?? '';
      const country = url.searchParams.get('country');
      const sector = url.searchParams.get('sector');

      let filtered = [...stocks];
      if (search) filtered = filtered.filter(s => s.symbol.toLowerCase().includes(search.toLowerCase()) || s.name.toLowerCase().includes(search.toLowerCase()));
      if (country) filtered = filtered.filter(s => s.country === country);
      if (sector) filtered = filtered.filter(s => s.sector === sector);

      const perPage = 10;
      const start = (page - 1) * perPage;
      return {
        items: filtered.slice(start, start + perPage),
        total: filtered.length,
        page,
        per_page: perPage,
        total_pages: Math.ceil(filtered.length / perPage),
      };
    },
  },
  {
    pattern: '**/api/analytics/filters**',
    handler: () => filters,
  },
  {
    pattern: '**/api/analytics/**',
    handler: () => ({ items: stocks.slice(0, 10), total: 20 }),
  },
  {
    pattern: '**/api/public/analytics**',
    handler: () => ({ items: stocks.slice(0, 10), total: 20 }),
  },
];
