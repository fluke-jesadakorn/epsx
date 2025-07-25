// Server-side configuration utility
export const config = {
  // Backend API URL - only available server-side
  getBackendUrl(): string {
    const url = process.env.API_URL || process.env.BACKEND_URL || 'http://localhost:8080';
    return url;
  },

  // Client-side should use relative URLs or API routes
  getApiUrl(): string {
    // For client-side, always use relative paths or Next.js API routes
    return '/api';
  },

  // Check if we're on the server side
  isServer(): boolean {
    return typeof window === 'undefined';
  }
};