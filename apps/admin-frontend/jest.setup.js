// Optional: configure or set up a testing framework before each test.
// If you delete this file, remove `setupFilesAfterEnv` from `jest.config.js`

// Used for __tests__/testing-library.js
// Learn more: https://github.com/testing-library/jest-dom
import '@testing-library/jest-dom'

// Mock Next.js router
jest.mock('next/navigation', () => ({
  useRouter() {
    return {
      push: jest.fn(),
      replace: jest.fn(),
      prefetch: jest.fn(),
      back: jest.fn(),
      forward: jest.fn(),
      refresh: jest.fn(),
    }
  },
  useSearchParams() {
    return new URLSearchParams()
  },
  usePathname() {
    return ''
  },
}))

// Mock Server Auth Actions
jest.mock('./lib/actions/server-auth', () => ({
  getCurrentUser: jest.fn(() => Promise.resolve(null)),
  getBearerToken: jest.fn(() => Promise.resolve(null)),
  loginAction: jest.fn(() => Promise.resolve({ success: false })),
  logoutAction: jest.fn(() => Promise.resolve()),
  isAuthenticated: jest.fn(() => Promise.resolve(false)),
  validateSession: jest.fn(() => Promise.resolve(null)),
}))

// Global test setup
beforeEach(() => {
  // Reset all mocks before each test
  jest.clearAllMocks()
})

// Suppress console errors in tests unless specifically testing them
const originalError = console.error
beforeAll(() => {
  console.error = (...args) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Warning: ReactDOM.render is no longer supported')
    ) {
      return
    }
    originalError.call(console, ...args)
  }
})

afterAll(() => {
  console.error = originalError
})