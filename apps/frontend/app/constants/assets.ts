import type { AssetInfo } from '@/types/payment'

export const supportedAssets: AssetInfo[] = [
  {
    currency: 'USDT_TRC20',
    name: 'Tether (TRC20)',
    symbol: 'USDT',
    chain: 'TRX',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: 'Typically 34 characters long with the capital letter "T"'
  },
  {
    currency: 'USDT_ERC20',
    name: 'Tether (ERC20)',
    symbol: 'USDT',
    chain: 'Ethereum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'USDT_BSC',
    name: 'Tether (BSC)',
    symbol: 'USDT',
    chain: 'Binance Smart Chain',
    decimals: 18,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'USDT_ARB',
    name: 'Tether (Arbitrum)',
    symbol: 'USDT',
    chain: 'Arbitrum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'USDC_ERC20',
    name: 'USD Coin (ERC20)',
    symbol: 'USDC',
    chain: 'Ethereum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'USDC_ARB',
    name: 'USD Coin (Arbitrum)',
    symbol: 'USDC',
    chain: 'Arbitrum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'BTC',
    name: 'Bitcoin',
    symbol: 'BTC',
    chain: 'Bitcoin',
    decimals: 8,
    depositThreshold: 0.0001,
    addressFormat: 'Between 14-74 characters'
  },
  {
    currency: 'ETH',
    name: 'Ethereum',
    symbol: 'ETH',
    chain: 'Ethereum',
    decimals: 18,
    depositThreshold: 0.001,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'TRX',
    name: 'TRON',
    symbol: 'TRX',
    chain: 'TRX',
    decimals: 18,
    depositThreshold: 15,
    addressFormat: 'Typically 34 characters long with the capital letter "T"'
  },
  {
    currency: 'BNB_BSC',
    name: 'Binance Coin (BSC)',
    symbol: 'BNB',
    chain: 'Binance Smart Chain',
    decimals: 18,
    depositThreshold: 0.01,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'DOGE',
    name: 'Dogecoin',
    symbol: 'DOGE',
    chain: 'Dogecoin',
    decimals: 8,
    depositThreshold: 5,
    addressFormat: 'Typically begin with a \'D\' and often 34 characters long'
  },
  {
    currency: 'LTC',
    name: 'Litecoin',
    symbol: 'LTC',
    chain: 'litecoin',
    decimals: 8,
    depositThreshold: 0.01,
    addressFormat: 'Between 14-74 characters'
  },
  {
    currency: 'BCH',
    name: 'Bitcoin Cash',
    symbol: 'BCH',
    chain: 'Bitcoin Cash',
    decimals: 8,
    depositThreshold: 0.01,
    addressFormat: 'Between 14-74 characters'
  }
]
