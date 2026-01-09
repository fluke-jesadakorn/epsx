// Server-side configuration utility - Modernized with centralized URL resolver
import { APIPath, Service, URL, URLContext } from '@/shared/utils/url-resolver';

export const config = {
  // Backend API URL - modernized with centralized URL resolver
  getBackendUrl(): string {
    return URL.get(Service.BACKEND, URLContext.SERVER);
  },

  // Client-side should use relative URLs or API routes
  getApiUrl(): string {
    // For client-side, always use relative paths or Next.js API routes
    return '/api';
  },

  // Get backend API base URL for direct API calls
  getBackendApiUrl(): string {
    return URL.api(Service.BACKEND, 'api', URLContext.SERVER);
  },

  // Get backend v1 API URL
  getBackendV1Url(): string {
    return URL.api(Service.BACKEND, 'api/v1', URLContext.SERVER);
  },

  // Get backend admin API URL
  getBackendAdminUrl(): string {
    return URL.api(Service.BACKEND, APIPath.ADMIN, URLContext.SERVER);
  },

  // Check if we're on the server side
  isServer(): boolean {
    return typeof window === 'undefined';
  },

  // Get port from environment
  getPort(): number {
    return process.env.PORT ? parseInt(process.env.PORT) : 3001;
  }
};