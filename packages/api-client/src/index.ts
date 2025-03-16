import { ApiClient } from './lib/api-client';

export type { ChatRequest, ChatResponse } from '@epsx/types';
export { ApiClient };

// Create default instance
export const apiClient = new ApiClient();

// Export types
export type { AxiosInstance, AxiosError } from 'axios';
