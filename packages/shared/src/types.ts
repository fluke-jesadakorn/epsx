// Define shared types and interfaces
export interface SharedConfig {
  environment: 'development' | 'production' | 'test'
  version: string
}

// Common response type
export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: string
}
