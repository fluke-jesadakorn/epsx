import '@testing-library/jest-dom'
import 'jest-environment-jsdom'

// Helper to ensure test environment variables are set
function setupTestEnv() {
  // Set test environment variables directly for Jest
  if (!process.env.NODE_ENV) process.env.NODE_ENV = 'test';
  if (!process.env.BACKEND_URL) process.env.BACKEND_URL = 'http://localhost:8080';
  if (!process.env.API_URL) process.env.API_URL = 'http://localhost:8080';
}

// Initialize test environment
setupTestEnv();

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

// Mock Next.js headers for Server Components
jest.mock('next/headers', () => ({
  cookies: jest.fn(() => ({
    get: jest.fn(),
    set: jest.fn(),
    delete: jest.fn(),
  })),
  headers: jest.fn(() => ({
    get: jest.fn(),
  })),
}))

// Mock Server Actions
jest.mock('server-only', () => ({}))

// Environment variables are setup by setupTestEnv() function above

// Mock fetch for Server Actions
global.fetch = jest.fn()

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  warn: jest.fn(),
  error: jest.fn(),
}

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(),
    removeListener: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
})

// Mock ResizeObserver
global.ResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}))

// Mock IntersectionObserver  
global.IntersectionObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}))

// Reset all mocks before each test
beforeEach(() => {
  jest.clearAllMocks()
})
