# Utils Directory Structure (Refactored)

This directory has been refactored for maximum brevity and maintainability. All utilities are now organized into 6 core files with short, memorable names.

## Core Files

### `env.ts` - Environment & Level utilities
- **Environment**: `isProd`, `isTest`, `isDev`, `env()`
- **Assets**: `asset()`, `defCur()`, `supCur()`, `apiUrl()`
- **Levels**: `lvlNum()`, `lvlName()`, `lvlFmt()`, `lvlNext()`, `lvlCol()`

### `fmt.ts` - Formatting utilities
- **Currency**: `cur(amt, cur?)`
- **Date**: `dt(date, fmt?)`
- **Percentage**: `pct(val, dec?)`
- **Price**: `prc(price)`
- **EPS Growth**: `epsGr(growth)`

### `stk.ts` - Stock processing utilities
- **Transform**: `xform(data)`, `xformPrice(data)`
- **Helpers**: `latest(stock)`, `avgEps(stock)`, `cmpLast(stock)`, `align(comp)`

### `tbl.ts` - Table utilities
- **Columns**: `cols(response)`
- **Rows**: `rows(response, level?)`
- **Masking**: `mask(value, level)`

### `cache.ts` - Cache utilities
- **Basic**: `set(key, val, ttl?)`, `get(key)`, `clear(key?)`
- **Stock**: `setStock(symbol, data)`, `getStock(symbol)`
- **Bulk**: `setBulk(stocks)`, `getBulk(symbols)`

### `util.ts` - General utilities
- **Functions**: `deb(fn, ms)`, `thr(fn, ms)`, `clone(obj)`, `id(pre?)`
- **Validation**: `mail(email)`, `phone(num)`
- **Text**: `trunc(text, len?)`
- **Storage**: `ls.get(key)`, `ls.set(key, val)`, `ls.del(key)`
- **Arrays**: `arr.uniq(arr)`, `arr.chunk(arr, n)`, `arr.group(arr, key)`
- **Objects**: `obj.pick(obj, keys)`, `obj.omit(obj, keys)`, `obj.isEmpty(obj)`
- **URLs**: `url.build(base, params)`, `url.parse(url)`

## Usage Examples

### Environment Detection
```typescript
import { isProd, env, asset, defCur, supCur } from '@/utils'

if (isProd) console.log('Production mode')
const currentEnv = env() // 'dev' | 'test' | 'prod'
const usdtConfig = asset('USDT_BSC')
const defaultCurrency = defCur()
const supported = supCur()
```

### Formatting
```typescript
import { cur, dt, pct, prc } from '@/utils'

const price = cur(1234.56, 'USD') // "$1,234.56"
const date = dt('2024-01-15') // "Jan 15, 2024"
const growth = pct(0.15) // "15.00%"
const stockPrice = prc(123.456) // "123.46"
```

### Stock Processing
```typescript
import { xform, latest, avgEps, align } from '@/utils'

const stocks = xform(apiData)
const lastQuarter = latest(stocks[0])
const avgGrowth = avgEps(stocks[0])
const alignment = align(comparisonData)
```

### Table Creation
```typescript
import { cols, rows } from '@/utils'

const columns = cols(apiResponse)
const data = rows(apiResponse, userLevel)
```

### Cache Usage
```typescript
import { set, get, setStock, getStock } from '@/utils'

set('key', data, 60000)
const cached = get('key')
setStock('AAPL', stockData)
const appleData = getStock('AAPL')
```

### General Utilities
```typescript
import { deb, clone, mail, ls, arr, obj, url } from '@/utils'

const debounced = deb(fn, 300)
const copy = clone(obj)
const valid = mail('test@example.com')
ls.set('key', value)
const unique = arr.uniq([1, 2, 2, 3])
const picked = obj.pick(obj, ['a', 'b'])
const apiUrl = url.build('/api', { q: 'search' })
```

## Migration Guide

### Old → New Mapping
- `environment.ts` → `env.ts`
- `level-utils.ts` → `env.ts`
- `processStocks/` → `stk.ts`
- `table/` → `tbl.ts`
- `cache/` → `cache.ts`

All old files are deprecated but still work with console warnings.

## File Size Comparison
- **Before**: 8+ files, ~500+ lines
- **After**: 6 files, ~300 lines total
- **Reduction**: ~40% smaller codebase
