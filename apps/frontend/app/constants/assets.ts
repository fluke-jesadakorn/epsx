import type { AssetInfo } from '@/types/payment'

export const supportedAssets: AssetInfo[] = [
  {
    currency: 'USDT_TRC20',
    chain: 'TRX',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: 'Typically 34 characters long with the capital letter "T"'
  },
  {
    currency: 'USDT_ERC20',
    chain: 'Ethereum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'USDT_BSC',
    chain: 'Binance Smart Chain',
    decimals: 18,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'USDT_ARB',
    chain: 'Arbitrum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'USDC_ERC20',
    chain: 'Ethereum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'USDC_ARB',
    chain: 'Arbitrum',
    decimals: 6,
    depositThreshold: 1,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'BTC',
    chain: 'Bitcoin',
    decimals: 8,
    depositThreshold: 0.0001,
    addressFormat: 'Between 14-74 characters'
  },
  {
    currency: 'ETH',
    chain: 'Ethereum',
    decimals: 18,
    depositThreshold: 0.001,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'TRX',
    chain: 'TRX',
    decimals: 18,
    depositThreshold: 15,
    addressFormat: 'Typically 34 characters long with the capital letter "T"'
  },
  {
    currency: 'BNB_BSC',
    chain: 'Binance Smart Chain',
    decimals: 18,
    depositThreshold: 0.01,
    addressFormat: '42-character string, beginning with \'0x\''
  },
  {
    currency: 'DOGE',
    chain: 'Dogecoin',
    decimals: 8,
    depositThreshold: 5,
    addressFormat: 'Typically begin with a \'D\' and often 34 characters long'
  },
  {
    currency: 'LTC',
    chain: 'litecoin',
    decimals: 8,
    depositThreshold: 0.01,
    addressFormat: 'Between 14-74 characters'
  },
  {
    currency: 'BCH',
    chain: 'Bitcoin Cash',
    decimals: 8,
    depositThreshold: 0.01,
    addressFormat: 'Between 14-74 characters'
  }
]
