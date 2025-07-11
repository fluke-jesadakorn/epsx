/**
 * Environment Detection and Configuration Utilities
 * Centralized environment management for EPSX project
 */

// Environment Detection
export const isProduction = process.env.NODE_ENV === 'production';
export const isTest = process.env.NODE_ENV === 'test';
export const isDevelopment = process.env.NODE_ENV === 'development';

// Environment type for better type safety
export type Environment = 'development' | 'test' | 'production';

export function getCurrentEnvironment(): Environment {
  if (isProduction) return 'production';
  if (isTest) return 'test';
  return 'development';
}

// Asset Configuration Types
export interface AssetConfig {
  chain: string;
  decimals: number;
  depositThreshold: number;
  addressFormat: string;
  faucet?: string; // Only available for testnet assets
}

export interface AssetMap {
  [currency: string]: AssetConfig;
}

// Supported MusePay Assets Configuration
export const SUPPORTED_ASSETS = {
  // Production/Mainnet Tokens
  mainnet: {
    'USDT_TRC20': {
      chain: 'TRX',
      decimals: 6,
      depositThreshold: 1,
      addressFormat: 'Typically 34 characters long with the capital letter "T"'
    },
    'USDT_ERC20': {
      chain: 'Ethereum',
      decimals: 6,
      depositThreshold: 1,
      addressFormat: '42-character string, beginning with \'0x\''
    },
    'USDT_BSC': {
      chain: 'Binance Smart Chain',
      decimals: 18,
      depositThreshold: 1,
      addressFormat: '42-character string, beginning with \'0x\''
    },
    'USDT_ARB': {
      chain: 'Arbitrum',
      decimals: 6,
      depositThreshold: 1,
      addressFormat: '42-character string, beginning with \'0x\''
    },
    'USDC_ERC20': {
      chain: 'Ethereum',
      decimals: 6,
      depositThreshold: 1,
      addressFormat: '42-character string, beginning with \'0x\''
    },
    'USDC_ARB': {
      chain: 'Arbitrum',
      decimals: 6,
      depositThreshold: 1,
      addressFormat: '42-character string, beginning with \'0x\''
    },
    'BTC': {
      chain: 'Bitcoin',
      decimals: 8,
      depositThreshold: 0.0001,
      addressFormat: 'Between 14-74 characters'
    },
    'ETH': {
      chain: 'Ethereum',
      decimals: 18,
      depositThreshold: 0.001,
      addressFormat: '42-character string, beginning with \'0x\''
    },
    'TRX': {
      chain: 'TRX',
      decimals: 18,
      depositThreshold: 15,
      addressFormat: 'Typically 34 characters long with the capital letter "T"'
    },
    'BNB_BSC': {
      chain: 'Binance Smart Chain',
      decimals: 18,
      depositThreshold: 0.01,
      addressFormat: '42-character string, beginning with \'0x\''
    },
    'DOGE': {
      chain: 'Dogecoin',
      decimals: 8,
      depositThreshold: 5,
      addressFormat: 'Typically begin with a \'D\' and often 34 characters long'
    },
    'LTC': {
      chain: 'litecoin',
      decimals: 8,
      depositThreshold: 0.01,
      addressFormat: 'Between 14-74 characters'
    },
    'BCH': {
      chain: 'Bitcoin Cash',
      decimals: 8,
      depositThreshold: 0.01,
      addressFormat: 'Between 14-74 characters'
    }
  } as AssetMap,
  
  // Testnet Tokens
  testnet: {
    'BTC_TEST': {
      chain: 'Bitcoin Testnet',
      decimals: 8,
      depositThreshold: 0.0001,
      addressFormat: 'Between 14-74 characters',
      faucet: 'https://coinfaucet.eu/en/btc-testnet/'
    },
    'ETH_TEST': {
      chain: 'Goerli Ethereum',
      decimals: 18,
      depositThreshold: 0.001,
      addressFormat: '42-character string, beginning with \'0x\'',
      faucet: 'https://goerlifaucet.com/'
    },
    'BNB_TEST': {
      chain: 'Binance Testnet',
      decimals: 18,
      depositThreshold: 0.01,
      addressFormat: '42-character string, beginning with \'0x\'',
      faucet: 'https://testnet.binance.org/faucet-smart'
    },
    'USDT_BSC_TEST': {
      chain: 'Binance Testnet',
      decimals: 18,
      depositThreshold: 1,
      addressFormat: '42-character string, beginning with \'0x\'',
      faucet: 'https://testnet.binance.org/faucet-smart'
    }
  } as AssetMap
};

/**
 * Get environment-appropriate asset configuration
 * @param currency - The currency symbol (e.g., 'USDT_BSC', 'BTC_TEST')
 * @returns Asset configuration or null if not found
 */
export function getAssetConfig(currency: string): AssetConfig | null {
  const assetMap = isProduction ? SUPPORTED_ASSETS.mainnet : SUPPORTED_ASSETS.testnet;
  return assetMap[currency] || null;
}

/**
 * Get environment-appropriate default currency
 * @returns Default currency for current environment
 */
export function getDefaultCurrency(): string {
  if (isProduction) {
    return 'USDT_BSC'; // Mainnet USDT on BSC
  }
  return 'USDT_BSC_TEST'; // Testnet USDT on BSC
}

/**
 * Get all supported currencies for current environment
 * @returns Array of supported currency symbols
 */
export function getSupportedCurrencies(): string[] {
  const assetMap = isProduction ? SUPPORTED_ASSETS.mainnet : SUPPORTED_ASSETS.testnet;
  return Object.keys(assetMap);
}

/**
 * Check if current environment is using testnet
 * @returns True if using testnet (development or test), false for production
 */
export function isTestnetEnvironment(): boolean {
  return !isProduction;
}

/**
 * Get environment-specific API URLs
 */
export const API_URLS = {
  musepay: {
    testnet: 'https://api.test.topay.mobi/v1',
    mainnet: 'https://api.topay.mobi/v1'
  }
} as const;

/**
 * Get MusePay API URL for current environment
 * @returns MusePay API URL
 */
export function getMusePayApiUrl(): string {
  return isProduction ? API_URLS.musepay.mainnet : API_URLS.musepay.testnet;
}

/**
 * Get environment-specific database names
 */
export const DATABASE_NAMES = {
  development: 'epsx',
  test: 'epsx_test',
  production: 'epsx_production'
} as const;

/**
 * Get database name for current environment
 * @returns Database name
 */
export function getDatabaseName(): string {
  const env = getCurrentEnvironment();
  return DATABASE_NAMES[env];
}

/**
 * Environment configuration summary
 */
export function getEnvironmentSummary() {
  const env = getCurrentEnvironment();
  
  return {
    environment: env,
    isProduction,
    isTest,
    isDevelopment,
    isTestnet: isTestnetEnvironment(),
    defaultCurrency: getDefaultCurrency(),
    supportedCurrencies: getSupportedCurrencies(),
    musePayApiUrl: getMusePayApiUrl(),
    databaseName: getDatabaseName(),
    assetCount: getSupportedCurrencies().length
  };
}

/**
 * Validate environment configuration
 * @returns Object with validation results
 */
export function validateEnvironmentConfig() {
  const issues: string[] = [];
  const warnings: string[] = [];
  
  // Check if required environment variables are set
  const requiredEnvVars = [
    'MUSEPAY_PARTNER_ID',
    'MUSEPAY_PRIVATE_KEY',
    'MUSEPAY_PUBLIC_KEY',
    'FIREBASE_PROJECT_ID'
  ];
  
  requiredEnvVars.forEach(envVar => {
    if (!process.env[envVar]) {
      issues.push(`Missing required environment variable: ${envVar}`);
    }
  });
  
  // Check if using production environment with test API
  if (isProduction && process.env.MUSEPAY_API_URL?.includes('test')) {
    issues.push('Production environment is using test MusePay API URL');
  }
  
  // Check if using development with production API
  if (!isProduction && process.env.MUSEPAY_API_URL && !process.env.MUSEPAY_API_URL.includes('test')) {
    warnings.push('Non-production environment is using production MusePay API URL');
  }
  
  return {
    isValid: issues.length === 0,
    issues,
    warnings,
    environment: getCurrentEnvironment()
  };
}
