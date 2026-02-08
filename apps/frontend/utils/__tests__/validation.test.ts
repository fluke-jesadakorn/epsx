// Frontend Utils Tests - Domain Layer (Pure Business Logic)
// Tests for validation utilities and business rules
// Clean Architecture: Domain Layer - No external dependencies

import { calculatePercentageChange, formatCurrency, isValidEmail } from '../index'

describe('Validation Utilities', () => {
  describe('Email Validation', () => {
    test('isValidEmail should validate correct email formats', () => {
      expect(isValidEmail('test@example.com')).toBe(true)
      expect(isValidEmail('user.name+tag@domain.co.uk')).toBe(true)
      expect(isValidEmail('user123@test-domain.com')).toBe(true)
      expect(isValidEmail('name@subdomain.domain.org')).toBe(true)
    })

    test('isValidEmail should reject invalid email formats', () => {
      expect(isValidEmail('invalid-email')).toBe(false)
      expect(isValidEmail('test@')).toBe(false)
      expect(isValidEmail('@domain.com')).toBe(false)
      expect(isValidEmail('test@domain.com')).toBe(false)
      expect(isValidEmail('')).toBe(false)
      expect(isValidEmail('test.domain.com')).toBe(false)
    })

    test('isValidEmail should handle edge cases', () => {
      expect(isValidEmail('a@b.co')).toBe(true) // Minimal valid email
      expect(isValidEmail('test@domain-name.com')).toBe(true) // Hyphenated domain
      expect(isValidEmail('test+filter@domain.com')).toBe(true) // Plus addressing
    })
  })

  describe('Permission Validation Business Rules', () => {
    const hasFeatureAccess = (userLevel: string, feature: string) => {
      const featureMap: Record<string, string[] | undefined> = {
        'user-basic-001': ['basic-analytics', 'stock-ranking'],
        'user-premium-002': ['basic-analytics', 'stock-ranking', 'advanced-analytics', 'export-data'],
        'moderator-standard-003': ['basic-analytics', 'stock-ranking', 'advanced-analytics', 'user-management'],
        'admin-full-004': ['*'] // All features
      }

      const userFeatures = featureMap[userLevel] ?? []
      return userFeatures.includes('*') || userFeatures.includes(feature)
    }

    test('basic user should have limited feature access', () => {
      expect(hasFeatureAccess('user-basic-001', 'basic-analytics')).toBe(true)
      expect(hasFeatureAccess('user-basic-001', 'stock-ranking')).toBe(true)
      expect(hasFeatureAccess('user-basic-001', 'advanced-analytics')).toBe(false)
      expect(hasFeatureAccess('user-basic-001', 'export-data')).toBe(false)
      expect(hasFeatureAccess('user-basic-001', 'user-management')).toBe(false)
    })

    test('premium user should have extended feature access', () => {
      expect(hasFeatureAccess('user-premium-002', 'basic-analytics')).toBe(true)
      expect(hasFeatureAccess('user-premium-002', 'stock-ranking')).toBe(true)
      expect(hasFeatureAccess('user-premium-002', 'advanced-analytics')).toBe(true)
      expect(hasFeatureAccess('user-premium-002', 'export-data')).toBe(true)
      expect(hasFeatureAccess('user-premium-002', 'user-management')).toBe(false)
    })

    test('moderator should have management features', () => {
      expect(hasFeatureAccess('moderator-standard-003', 'user-management')).toBe(true)
      expect(hasFeatureAccess('moderator-standard-003', 'advanced-analytics')).toBe(true)
      expect(hasFeatureAccess('moderator-standard-003', 'export-data')).toBe(false)
    })

    test('admin should have access to all features', () => {
      expect(hasFeatureAccess('admin-full-004', 'any-feature')).toBe(true)
      expect(hasFeatureAccess('admin-full-004', 'basic-analytics')).toBe(true)
      expect(hasFeatureAccess('admin-full-004', 'user-management')).toBe(true)
      expect(hasFeatureAccess('admin-full-004', 'system-configuration')).toBe(true)
    })

    test('unknown user level should have no access', () => {
      expect(hasFeatureAccess('unknown-level', 'basic-analytics')).toBe(false)
      expect(hasFeatureAccess('', 'basic-analytics')).toBe(false)
    })
  })
})

describe('Formatting Utilities', () => {
  describe('Currency Formatting', () => {
    test('formatCurrency should format USD correctly', () => {
      expect(formatCurrency(1234.56)).toBe('$1,234.56')
      expect(formatCurrency(0)).toBe('$0.00')
      expect(formatCurrency(1000000)).toBe('$1,000,000.00')
      expect(formatCurrency(0.99)).toBe('$0.99')
    })

    test('formatCurrency should handle negative values', () => {
      expect(formatCurrency(-1234.56)).toBe('-$1,234.56')
      expect(formatCurrency(-0.50)).toBe('-$0.50')
    })

    test('formatCurrency should handle edge cases', () => {
      expect(formatCurrency(null as unknown as number)).toBe('$0.00')
      expect(formatCurrency(undefined as unknown as number)).toBe('$0.00')
      expect(formatCurrency(NaN)).toBe('$0.00')
      expect(formatCurrency(Infinity)).toBe('$0.00')
    })
  })

  describe('Percentage Change Calculations', () => {
    test('calculatePercentageChange should calculate correctly', () => {
      expect(calculatePercentageChange(100, 110)).toBeCloseTo(10, 2)
      expect(calculatePercentageChange(100, 90)).toBeCloseTo(-10, 2)
      expect(calculatePercentageChange(110, 100)).toBeCloseTo(-9.09, 2)
      expect(calculatePercentageChange(50, 75)).toBeCloseTo(50, 2)
    })

    test('calculatePercentageChange should handle zero values', () => {
      expect(calculatePercentageChange(0, 100)).toBe(100)
      expect(calculatePercentageChange(100, 0)).toBe(-100)
      expect(calculatePercentageChange(0, 0)).toBe(0)
    })

    test('calculatePercentageChange should handle edge cases', () => {
      expect(calculatePercentageChange(100, 100)).toBe(0)
      expect(calculatePercentageChange(-100, -50)).toBeCloseTo(50, 2)
      expect(calculatePercentageChange(-50, -100)).toBeCloseTo(-100, 2)
    })
  })
})

describe('Input Validation Business Rules', () => {
  describe('Stock Symbol Validation', () => {
    const validateStockSymbol = (symbol: string): boolean => {
      if (!symbol || typeof symbol !== 'string') { return false }
      // Stock symbols: 1-5 uppercase letters
      const symbolRegex = /^[A-Z]{1,5}$/
      return symbolRegex.test(symbol.trim())
    }

    test('should validate correct stock symbols', () => {
      expect(validateStockSymbol('AAPL')).toBe(true)
      expect(validateStockSymbol('GOOGL')).toBe(true)
      expect(validateStockSymbol('MSFT')).toBe(true)
      expect(validateStockSymbol('A')).toBe(true)
      expect(validateStockSymbol('TSLA')).toBe(true)
    })

    test('should reject invalid stock symbols', () => {
      expect(validateStockSymbol('aapl')).toBe(false) // lowercase
      expect(validateStockSymbol('TOOLONG')).toBe(false) // too long
      expect(validateStockSymbol('')).toBe(false) // empty
      expect(validateStockSymbol('123')).toBe(false) // numbers
      expect(validateStockSymbol('AA-PL')).toBe(false) // special chars
      expect(validateStockSymbol(' AAPL ')).toBe(false) // with spaces
    })
  })

  describe('Amount Validation', () => {
    const validateAmount = (amount: unknown): boolean => {
      if (amount === null || amount === undefined || amount === '') { return false }
      const num = Number(amount)
      return !isNaN(num) && isFinite(num) && num >= 0
    }

    test('should validate correct amounts', () => {
      expect(validateAmount(100)).toBe(true)
      expect(validateAmount('100')).toBe(true)
      expect(validateAmount(0)).toBe(true)
      expect(validateAmount(999.99)).toBe(true)
      expect(validateAmount('0.01')).toBe(true)
    })

    test('should reject invalid amounts', () => {
      expect(validateAmount(-100)).toBe(false) // negative
      expect(validateAmount('abc')).toBe(false) // non-numeric
      expect(validateAmount('')).toBe(false) // empty
      expect(validateAmount(null)).toBe(false) // null
      expect(validateAmount(undefined)).toBe(false) // undefined
      expect(validateAmount(NaN)).toBe(false) // NaN
      expect(validateAmount(Infinity)).toBe(false) // Infinity
    })
  })
})