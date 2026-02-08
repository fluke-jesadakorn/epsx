// Frontend Utils Tests - Domain Layer (Pure Business Logic)
// Tests for financial calculations and business logic utilities
// Clean Architecture: Domain Layer - No external dependencies

import { 
  calculateEPSGrowth, 
  determineStockTrend, 
  calculateMovingAverage 
} from '../financial-calculations'

describe('Financial Calculations', () => {
  describe('EPS Growth Calculations', () => {
    test('calculateEPSGrowth should calculate correct growth rate', () => {
      const quarters = [1.0, 1.1, 1.21, 1.33]
      const growth = calculateEPSGrowth(quarters)
      expect(growth).toBeCloseTo(33.0, 1)
    })

    test('calculateEPSGrowth should handle zero/negative values', () => {
      const quarters = [0, 1.0, 1.1, 1.21]
      const growth = calculateEPSGrowth(quarters)
      expect(growth).toBeCloseTo(0, 1)
    })

    test('calculateEPSGrowth should handle single quarter', () => {
      const quarters = [1.0]
      const growth = calculateEPSGrowth(quarters)
      expect(growth).toBe(0)
    })

    test('calculateEPSGrowth should handle empty array', () => {
      const quarters: number[] = []
      const growth = calculateEPSGrowth(quarters)
      expect(growth).toBe(0)
    })
  })

  describe('Stock Trend Analysis', () => {
    test('determineStockTrend should identify upward trend', () => {
      const prices = [100, 105, 110, 115, 120]
      const trend = determineStockTrend(prices)
      expect(trend).toBe('upward')
    })

    test('determineStockTrend should identify downward trend', () => {
      const prices = [120, 115, 110, 105, 100]
      const trend = determineStockTrend(prices)
      expect(trend).toBe('downward')
    })

    test('determineStockTrend should identify sideways trend', () => {
      const prices = [100, 102, 98, 101, 99]
      const trend = determineStockTrend(prices)
      expect(trend).toBe('sideways')
    })

    test('determineStockTrend should handle insufficient data', () => {
      const prices = [100]
      const trend = determineStockTrend(prices)
      expect(trend).toBe('sideways')
    })
  })

  describe('Moving Average Calculations', () => {
    test('calculateMovingAverage should calculate correct SMA', () => {
      const prices = [10, 12, 14, 16, 18]
      const sma = calculateMovingAverage(prices, 5)
      expect(sma).toBe(14)
    })

    test('calculateMovingAverage should handle insufficient data', () => {
      const prices = [10, 12]
      const sma = calculateMovingAverage(prices, 5)
      expect(sma).toBe(11) // Uses available data
    })

    test('calculateMovingAverage should handle empty array', () => {
      const prices: number[] = []
      const sma = calculateMovingAverage(prices, 5)
      expect(sma).toBe(0)
    })

    test('calculateMovingAverage should handle rolling window', () => {
      const prices = [10, 12, 14, 16, 18, 20]
      const sma = calculateMovingAverage(prices, 3)
      expect(sma).toBe(18) // Average of last 3: (16+18+20)/3
    })
  })
})

describe('Stock Classification Business Rules', () => {
  describe('Growth Classification', () => {
    const classifyByGrowth = (epsGrowth: number) => {
      if (epsGrowth > 20) {return 'high-growth'}
      if (epsGrowth > 10) {return 'moderate-growth'}
      if (epsGrowth > 0) {return 'low-growth'}
      return 'declining'
    }

    test('should classify high growth stocks', () => {
      expect(classifyByGrowth(25)).toBe('high-growth')
      expect(classifyByGrowth(50)).toBe('high-growth')
    })

    test('should classify moderate growth stocks', () => {
      expect(classifyByGrowth(15)).toBe('moderate-growth')
      expect(classifyByGrowth(10.1)).toBe('moderate-growth')
    })

    test('should classify low growth stocks', () => {
      expect(classifyByGrowth(5)).toBe('low-growth')
      expect(classifyByGrowth(0.1)).toBe('low-growth')
    })

    test('should classify declining stocks', () => {
      expect(classifyByGrowth(-5)).toBe('declining')
      expect(classifyByGrowth(0)).toBe('declining')
    })
  })

  describe('Volatility Classification', () => {
    const classifyByVolatility = (prices: number[]) => {
      if (prices.length < 2) {return 'low-volatility'}
      
      const volatility = Math.sqrt(
        prices.reduce((sum, price, i, arr) => {
          if (i === 0) {return 0}
          const change = (price - arr[i - 1]) / arr[i - 1]
          return sum + Math.pow(change, 2)
        }, 0) / (prices.length - 1)
      )

      if (volatility > 0.05) {return 'high-volatility'}
      if (volatility > 0.02) {return 'moderate-volatility'}
      return 'low-volatility'
    }

    test('should classify high volatility stocks', () => {
      const highVolPrices = [100, 120, 90, 130, 80]
      expect(classifyByVolatility(highVolPrices)).toBe('high-volatility')
    })

    test('should classify low volatility stocks', () => {
      const lowVolPrices = [100, 101, 102, 101, 100]
      expect(classifyByVolatility(lowVolPrices)).toBe('low-volatility')
    })

    test('should handle insufficient data', () => {
      const singlePrice = [100]
      expect(classifyByVolatility(singlePrice)).toBe('low-volatility')
    })
  })
})