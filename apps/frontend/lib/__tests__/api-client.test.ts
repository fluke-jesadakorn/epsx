// Frontend Lib Tests - Infrastructure Layer (External Dependencies)
// Tests for API clients, external service integrations
// Clean Architecture: Infrastructure Layer - Tests external integrations with mocked responses

import { fetchStockData, fetchUserProfile, updateUserProfile } from '../api-client'

// Mock fetch globally
global.fetch = jest.fn()
const mockFetch = fetch as jest.MockedFunction<typeof fetch>

describe('API Client', () => {
  beforeEach(() => {
    mockFetch.mockClear()
  })

  describe('Stock Data API', () => {
    test('fetchStockData should handle successful response', async () => {
      const mockStockData = {
        symbol: 'AAPL',
        price: 150.25,
        change: 2.5,
        eps: 6.05
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockStockData
      } as Response)

      const result = await fetchStockData('AAPL')
      
      expect(result).toEqual(mockStockData)
      expect(mockFetch).toHaveBeenCalledWith('/api/stocks/AAPL', {
        headers: {
          'Content-Type': 'application/json',
          'Authorization': expect.any(String)
        }
      })
    })

    test('fetchStockData should handle network errors', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      await expect(fetchStockData('AAPL')).rejects.toThrow('Network error')
    })

    test('fetchStockData should handle HTTP errors', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found'
      } as Response)

      await expect(fetchStockData('INVALID')).rejects.toThrow('Failed to fetch stock data: 404')
    })
  })

  describe('User Profile API', () => {
    test('fetchUserProfile should return user data', async () => {
      const mockUser = {
        id: '123',
        email: 'test@example.com',
        permissionProfile: 'user-premium-002',
        subscriptionStatus: 'active'
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockUser
      } as Response)

      const result = await fetchUserProfile('123')
      
      expect(result).toEqual(mockUser)
      expect(mockFetch).toHaveBeenCalledWith('/api/users/123')
    })

    test('updateUserProfile should send PUT request with data', async () => {
      const updateData = { email: 'newemail@example.com' }
      
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true })
      } as Response)

      await updateUserProfile('123', updateData)

      expect(mockFetch).toHaveBeenCalledWith('/api/users/123', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': expect.any(String)
        },
        body: JSON.stringify(updateData)
      })
    })
  })

  describe('Authentication Headers', () => {
    test('API calls should include authentication headers', async () => {
      // Mock localStorage.getItem to return a token
      Storage.prototype.getItem = jest.fn(() => 'mock-jwt-token')

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({})
      } as Response)

      await fetchStockData('AAPL')

      expect(mockFetch).toHaveBeenCalledWith(expect.any(String), expect.objectContaining({
        headers: expect.objectContaining({
          'Authorization': 'Bearer mock-jwt-token'
        })
      }))
    })
  })

  describe('Error Handling', () => {
    test('should handle malformed JSON responses', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => { throw new Error('Invalid JSON') }
      } as Response)

      await expect(fetchStockData('AAPL')).rejects.toThrow('Invalid JSON')
    })

    test('should handle timeout scenarios', async () => {
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Request timeout')), 100)
      })

      mockFetch.mockImplementationOnce(() => timeoutPromise)

      await expect(fetchStockData('AAPL')).rejects.toThrow('Request timeout')
    })
  })
})

describe('API Retry Logic', () => {
  test('should retry failed requests with exponential backoff', async () => {
    // First two calls fail, third succeeds
    mockFetch
      .mockRejectedValueOnce(new Error('Network error'))
      .mockRejectedValueOnce(new Error('Network error'))
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ symbol: 'AAPL', price: 150 })
      } as Response)

    const result = await fetchStockData('AAPL')
    
    expect(result).toEqual({ symbol: 'AAPL', price: 150 })
    expect(mockFetch).toHaveBeenCalledTimes(3)
  })

  test('should give up after maximum retry attempts', async () => {
    mockFetch.mockRejectedValue(new Error('Persistent network error'))

    await expect(fetchStockData('AAPL')).rejects.toThrow('Persistent network error')
    expect(mockFetch).toHaveBeenCalledTimes(3) // Initial + 2 retries
  })
})