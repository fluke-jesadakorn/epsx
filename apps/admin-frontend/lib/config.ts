// Server-side configuration utility
import { env } from '@/config/env';

export const config = {
  // Backend API URL - only available server-side
  getBackendUrl(): string {
    // Use consolidated environment configuration
    return env.BACKEND_URL;
  },

  // Client-side should use relative URLs or API routes
  getApiUrl(): string {
    // For client-side, always use relative paths or Next.js API routes
    return '/api';
  },

  // Get backend API base URL for direct API calls
  getBackendApiUrl(): string {
    return `${this.getBackendUrl()}/api`;
  },

  // Get backend v1 API URL
  getBackendV1Url(): string {
    return `${this.getBackendUrl()}/api/v1`;
  },

  // Get backend admin API URL
  getBackendAdminUrl(): string {
    return `${this.getBackendUrl()}/api/admin`;
  },

  // Check if we're on the server side
  isServer(): boolean {
    return typeof window === 'undefined';
  },

  // Get port from environment
  getPort(): number {
    return env.PORT;
  }
};