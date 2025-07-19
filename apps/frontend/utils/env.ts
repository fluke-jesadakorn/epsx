// Environment & Level utilities
export const isProd = process.env.NODE_ENV === 'production'
export const isTest = process.env.NODE_ENV === 'test'
export const isDev = process.env.NODE_ENV === 'development'

export type Env = 'dev' | 'test' | 'prod'

export const env = (): Env => isProd ? 'prod' : isTest ? 'test' : 'dev'

// Asset config
export interface AssetCfg {
  chain: string
  decimals: number
  depositThreshold: number
  addressFormat: string
  faucet?: string
}

const ASSETS = {
  mainnet: {
    USDT_TRC20: { chain: 'TRX', decimals: 6, depositThreshold: 1, addressFormat: '34 chars with T' },
    USDT_ERC20: { chain: 'Ethereum', decimals: 6, depositThreshold: 1, addressFormat: '42 chars with 0x' },
    USDT_BSC: { chain: 'BSC', decimals: 18, depositThreshold: 1, addressFormat: '42 chars with 0x' },
    USDT_ARB: { chain: 'Arbitrum', decimals: 6, depositThreshold: 1, addressFormat: '42 chars with 0x' },
    USDC_ERC20: { chain: 'Ethereum', decimals: 6, depositThreshold: 1, addressFormat: '42 chars with 0x' },
    USDC_ARB: { chain: 'Arbitrum', decimals: 6, depositThreshold: 1, addressFormat: '42 chars with 0x' },
    BTC: { chain: 'Bitcoin', decimals: 8, depositThreshold: 0.0001, addressFormat: '14-74 chars' },
    ETH: { chain: 'Ethereum', decimals: 18, depositThreshold: 0.001, addressFormat: '42 chars with 0x' },
    TRX: { chain: 'TRX', decimals: 18, depositThreshold: 15, addressFormat: '34 chars with T' },
    BNB_BSC: { chain: 'BSC', decimals: 18, depositThreshold: 0.01, addressFormat: '42 chars with 0x' },
    DOGE: { chain: 'Dogecoin', decimals: 8, depositThreshold: 5, addressFormat: '34 chars with D' },
    LTC: { chain: 'Litecoin', decimals: 8, depositThreshold: 0.01, addressFormat: '14-74 chars' },
    BCH: { chain: 'Bitcoin Cash', decimals: 8, depositThreshold: 0.01, addressFormat: '14-74 chars' }
  },
  testnet: {
    BTC_TEST: { chain: 'Bitcoin Testnet', decimals: 8, depositThreshold: 0.0001, addressFormat: '14-74 chars', faucet: 'https://coinfaucet.eu/en/btc-testnet/' },
    ETH_TEST: { chain: 'Goerli', decimals: 18, depositThreshold: 0.001, addressFormat: '42 chars with 0x', faucet: 'https://goerlifaucet.com/' },
    BNB_TEST: { chain: 'BSC Testnet', decimals: 18, depositThreshold: 0.01, addressFormat: '42 chars with 0x', faucet: 'https://testnet.binance.org/faucet-smart' },
    USDT_BSC_TEST: { chain: 'BSC Testnet', decimals: 18, depositThreshold: 1, addressFormat: '42 chars with 0x', faucet: 'https://testnet.binance.org/faucet-smart' }
  }
}

export const asset = (cur: string): AssetCfg | null => {
  const assets = isProd ? ASSETS.mainnet : ASSETS.testnet
  return (assets as any)[cur] || null
}
export const defCur = (): string => isProd ? 'USDT_BSC' : 'USDT_BSC_TEST'
export const supCur = (): string[] => Object.keys(isProd ? ASSETS.mainnet : ASSETS.testnet)
export const apiUrl = (): string => isProd ? 'https://api.topay.mobi/v1' : 'https://api.test.topay.mobi/v1'
export const dbName = (): string => ({ dev: 'epsx', test: 'epsx_test', prod: 'epsx_production' } as any)[env()]

// Level utilities
const LVL_MAP = { BRONZE: 1, SILVER: 2, GOLD: 3, PLATINUM: 4, DIAMOND: 5, VIP: 6 }
const LVL_NAMES = { BRONZE: 'Bronze', SILVER: 'Silver', GOLD: 'Gold', PLATINUM: 'Platinum', DIAMOND: 'Diamond', VIP: 'VIP' }
const LVL_COLS = { BRONZE: 'text-amber-600', SILVER: 'text-slate-400', GOLD: 'text-yellow-500', PLATINUM: 'text-purple-500', DIAMOND: 'text-blue-500', VIP: 'text-red-500' }

export const lvlNum = (lvl: string): number => (LVL_MAP as any)[lvl] || 0
export const lvlName = (lvl: string): string => (LVL_NAMES as any)[lvl] || 'Bronze'
export const lvlFmt = (lvl: string): string => `Level ${lvlNum(lvl)}`
export const lvlNext = (lvl: string): string => `Level ${lvlNum(lvl) + 1}`
export const lvlCol = (lvl: string): string => (LVL_COLS as any)[lvl] || 'text-gray-500'

// Backward compatibility
export const getCurrentEnvironment = env
export const getAssetConfig = asset
export const getDefaultCurrency = defCur
export const getSupportedCurrencies = supCur
export const getMusePayApiUrl = apiUrl
export const getDatabaseName = dbName
export const getLevelNumber = lvlNum
export const getLevelName = lvlName
export const formatLevelAsNumber = lvlFmt
export const getNextLevelName = lvlNext
export const getLevelColor = lvlCol
