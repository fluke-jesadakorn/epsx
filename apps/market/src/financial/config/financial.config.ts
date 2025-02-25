// Scraping configuration has been moved to scheduler service
// See apps/scheduler-service/.env for configuration

export const FinancialConfig = {
  cache: {
    expiry: 3600000, // 1 hour
  },
  api: {
    // Kept for possible future API endpoints
    baseUrl: "https://stockanalysis.com/quote",
  },
};
