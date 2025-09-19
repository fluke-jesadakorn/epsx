// Main API client module - re-exports consolidated API functionality
export * from './api';

// Export utilities that some components expect
export { isApiError } from '@/types/api';

// Export factory function for creating API client instances  
export const createApiClient = () => {
  const { apiClient } = require('./api');
  return apiClient;
};