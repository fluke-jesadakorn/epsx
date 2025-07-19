// DEPRECATED: Use env.ts instead
console.warn('environment.ts is deprecated. Use env.ts instead')

export {
  getCurrentEnvironment,
  getAssetConfig,
  getDefaultCurrency,
  getSupportedCurrencies,
  isProd as isProduction,
  isTest,
  isDev as isDevelopment,
  apiUrl as getMusePayApiUrl
} from './env'
