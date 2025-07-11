# Utils Directory Structure

This directory contains utility functions and helper modules organized by:

## environment.ts
- **Environment Detection**: Centralized environment management for EPSX project
- **MusePay Asset Configuration**: Testnet vs mainnet token configuration
- **Functions**:
  - `getCurrentEnvironment()` - Get current environment (development|test|production)
  - `getAssetConfig(currency)` - Get asset configuration for specific currency
  - `getDefaultCurrency()` - Get environment-appropriate default currency
  - `getSupportedCurrencies()` - Get all supported currencies for current environment
  - `getMusePayApiUrl()` - Get MusePay API URL for current environment
  - `validateEnvironmentConfig()` - Validate environment configuration
  - `getEnvironmentSummary()` - Get complete environment summary

## auth/
- Authentication-related utilities
- Example: auth.ts

## supabase/
- Supabase client/server utilities
- Example: client.ts, server.ts

## table/
- Table-related utilities
- Example: tableUtils.ts

## cache/
- Caching utilities and helpers

## processStocks/
- Stock processing utilities

## transformers/
- Data transformation utilities

## Usage Examples

### Environment Detection
```typescript
import { 
  getCurrentEnvironment, 
  isProduction, 
  getDefaultCurrency,
  getSupportedCurrencies 
} from '@/utils/environment';

// Check current environment
const env = getCurrentEnvironment(); // 'development' | 'test' | 'production'

// Get supported currencies for current environment
const currencies = getSupportedCurrencies();
// Dev/Test: ['BTC_TEST', 'ETH_TEST', 'BNB_TEST', 'USDT_BSC_TEST']
// Prod: ['USDT_TRC20', 'USDT_ERC20', 'USDT_BSC', 'BTC', 'ETH', ...]

// Get default currency
const defaultCurrency = getDefaultCurrency();
// Dev/Test: 'USDT_BSC_TEST'
// Prod: 'USDT_BSC'
```

### Asset Configuration
```typescript
import { getAssetConfig } from '@/utils/environment';

// Get asset configuration
const assetConfig = getAssetConfig('USDT_BSC');
// Returns: { 
//   chain: 'Binance Smart Chain', 
//   decimals: 18, 
//   depositThreshold: 1,
//   addressFormat: '42-character string, beginning with \'0x\''
// }

// For testnet assets, faucet URLs are included
const testAsset = getAssetConfig('USDT_BSC_TEST');
// Returns: { 
//   chain: 'Binance Testnet', 
//   decimals: 18, 
//   depositThreshold: 1,
//   addressFormat: '42-character string, beginning with \'0x\'',
//   faucet: 'https://testnet.binance.org/faucet-smart'
// }
```

## TODO:
- Add JSDoc documentation for all utility functions
- Implement error handling patterns
- Add unit tests for utility functions
- Create a centralized error logging system
- Add environment validation tests
- Create environment-specific configuration validators
