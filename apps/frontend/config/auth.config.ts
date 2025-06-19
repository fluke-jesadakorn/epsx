export const defaultAuthConfig = {
  apiUrl: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000',
  endpoints: {
    login: '/auth/login',
    register: '/auth/register',
    logout: '/auth/logout',
    refresh: '/auth/refresh',
    me: '/auth/me'
  },
  tokenKey: 'authToken',
  refreshTokenKey: 'refreshToken',
  tokenType: 'Bearer'
};
