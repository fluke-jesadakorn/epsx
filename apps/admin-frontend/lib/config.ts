// Server-side configuration utility
export const config = {
  // Backend API URL - only available server-side
  getBackendUrl(): string {
    // Use environment variables to get backend URL
    const backendUrl = process.env.BACKEND_URL || process.env.API_URL || 'http://localhost:8080';
    return backendUrl;
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
    return parseInt(process.env.PORT || '3001', 10);
  }
};