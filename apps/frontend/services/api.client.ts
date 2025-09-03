// Updated to use the refactored API client packages
import { apiClient } from '@/lib/api-client';

// Server actions for server-side operations
import {
  getCurrentUser,
  checkFeatureAccess,
  getTransactionHistory,
  signOut,
  getSession
} from '@/lib/server-actions';

// Legacy wrapper class for backward compatibility
class ApiService {
  async get<T>(_endpoint: string): Promise<T> {
    // This method is deprecated - use apiClient domain-specific methods instead
    throw new Error('Direct API calls are deprecated. Use apiClient.domain.method() or server actions instead.');
  }

  async post<T>(_endpoint: string, _data?: any): Promise<T> {
    // This method is deprecated - use apiClient domain-specific methods instead
    throw new Error('Direct API calls are deprecated. Use apiClient.domain.method() or server actions instead.');
  }

  async put<T>(_endpoint: string, _data?: any): Promise<T> {
    // This method is deprecated - use apiClient domain-specific methods instead
    throw new Error('Direct API calls are deprecated. Use apiClient.domain.method() or server actions instead.');
  }

  async delete<T>(_endpoint: string): Promise<T> {
    // This method is deprecated - use apiClient domain-specific methods instead
    throw new Error('Direct API calls are deprecated. Use apiClient.domain.method() or server actions instead.');
  }
}

export const api = new ApiService();

// Domain-specific API endpoints using the new client structure
export const stockApi = {
  getStocks: async () => {
    const result = await apiClient.get('/api/v1/analytics/eps-rankings');
    return result;
  },
  
  getSymbols: async (query?: string) => {
    const result = await apiClient.get('/api/v1/analytics/search', { query: query || '' });
    return result;
  },
  
  getStock: async (symbol: string) => {
    return await apiClient.get(`/api/v1/analytics/stocks/${symbol}`);
  },
  
  searchStocks: async (query: string, limit?: number) => {
    return await apiClient.get('/api/v1/analytics/search', { query, limit });
  },
  
  getStockHistory: async (symbol: string) => {
    return await apiClient.get(`/api/v1/analytics/stocks/${symbol}/financials`);
  },

  // Watchlist operations
  getWatchlist: async () => {
    return await apiClient.get('/api/v1/analytics/watchlist');
  },

  addToWatchlist: async (symbol: string, notes?: string) => {
    return await apiClient.post('/api/v1/analytics/watchlist', { symbol, notes });
  },

  removeFromWatchlist: async (symbol: string) => {
    return await apiClient.delete(`/api/v1/analytics/watchlist/${symbol}`);
  },
};

export const rankingApi = {
  getRankings: async (category?: string, limit?: number, page?: number) => {
    return await apiClient.get('/api/v1/analytics/eps-rankings', { category, limit, page });
  },
  
  getUserRanking: async (_userId: string) => {
    // This would need to be implemented as a new server action
    throw new Error('getUserRanking needs to be implemented as a server action');
  },
  
  updateRanking: async (_data: any) => {
    // This would need to be implemented as a new server action
    throw new Error('updateRanking needs to be implemented as a server action');
  },
};

export const paymentApi = {
  createPayment: async (data: {
    planId: string;
    paymentMethod: string;
    billingAddress?: any;
    couponCode?: string;
  }) => {
    return await apiClient.post('/api/payments', data);
  },
  
  getPaymentStatus: async (paymentIntentId: string) => {
    return await apiClient.get(`/api/payments/status/${paymentIntentId}`);
  },
  
  verifyPayment: async (paymentId: string) => {
    return await apiClient.post(`/api/payments/verify/${paymentId}`);
  },
  
  getPlans: async () => {
    return await apiClient.get('/api/payments/plans');
  },

  getPlan: async (id: string) => {
    return await apiClient.get(`/api/payments/plans/${id}`);
  },

  getUserSubscription: async () => {
    return await apiClient.get('/api/payments/subscription');
  },

  cancelSubscription: async (subscriptionId: string) => {
    return await apiClient.delete(`/api/payments/subscription/${subscriptionId}`);
  },

  getPaymentHistory: async (page: number = 1, limit: number = 10) => {
    return await apiClient.get('/api/payments/history', { page, limit });
  },
  
  getCryptoDepositAddress: async () => {
    // This would need to be implemented as a new server action
    throw new Error('getCryptoDepositAddress needs to be implemented as a server action');
  },
  
  getQrCode: async (_paymentId: string) => {
    // This would need to be implemented as a new server action
    throw new Error('getQrCode needs to be implemented as a server action');
  },
};

export const userApi = {
  getProfile: async () => {
    return await apiClient.get('/api/user/profile');
  },
  
  updateProfile: async (data: {
    name?: string;
    displayName?: string;
    avatar?: string;
    preferences?: any;
  }) => {
    return await apiClient.put('/api/user/profile', data);
  },
  
  getUserData: async () => {
    return await apiClient.get('/api/user/profile');
  },
  
  updateUserData: async (data: {
    name?: string;
    displayName?: string;
    avatar?: string;
    preferences?: any;
  }) => {
    return await apiClient.put('/api/user/profile', data);
  },
  
  login: async (email: string, password: string) => {
    return await apiClient.post('/oauth/token', { email, password, grant_type: 'password' });
  },

  register: async (data: { email: string; password: string; name?: string; package_tier?: string }) => {
    return await apiClient.post('/api/auth/register', data);
  },

  logout: async () => {
    return await apiClient.post('/api/auth/logout');
  },

  changePassword: async (currentPassword: string, newPassword: string) => {
    return await apiClient.post('/api/user/change-password', { currentPassword, newPassword });
  },

  resetPassword: async (email: string) => {
    return await apiClient.post('/api/auth/reset-password', { email });
  },
  
  getUserById: async (_id: string) => {
    // This would need to be implemented as a new server action for admin users
    throw new Error('getUserById needs to be implemented as a server action');
  },
  
  listUsers: async () => {
    // This would need to be implemented as a new server action for admin users
    throw new Error('listUsers needs to be implemented as a server action');
  },
  
  deleteUser: async (_id: string) => {
    // This would need to be implemented as a new server action for admin users
    throw new Error('deleteUser needs to be implemented as a server action');
  },
};

// Simple feature access API - replaces complex permissions system
export const featureAccessApi = {
  checkFeatureAccess: async (feature: string) => {
    return await checkFeatureAccess(feature); // Use server action with simple role system
  },

  checkRankingAccess: async () => {
    return await checkFeatureAccess('analytics'); // Use server action
  },

  // Legacy permission check - maps to feature access
  checkPermission: async (permission: string) => {
    // Map legacy permissions to features
    const featureMap: Record<string, string> = {
      'users.view': 'view_eps',
      'dashboard.view': 'view_eps',
      'analytics.view': 'view_eps',
      'analytics.export': 'export_data',
      'realtime': 'realtime',
      'profile': 'profile',
      'notifications': 'notifications',
      'billing': 'billing',
      'advanced_filters': 'advanced_filters'
    };

    const feature = featureMap[permission] || permission;
    return await checkFeatureAccess(feature);
  },

  // Get user's role-based features
  getUserFeatures: async () => {
    // This would need to be implemented to get user's current role and return features
    // For now, return basic feature set
    return ['view_eps']; // Default to guest features
  },
};

// Legacy export for backward compatibility
export const legacyApiClient = {
  // Auth methods using new structure
  getCurrentUser: () => getCurrentUser(),
  updateProfile: (data: any) => userApi.updateProfile(data),
  login: (credentials: any) => userApi.login(credentials.email, credentials.password),
  logout: () => signOut(),
  register: (data: any) => userApi.register(data),

  // Stock methods using new structure
  getBatchStocks: () => stockApi.getStocks(),
  getStockData: (symbol: string) => stockApi.getStock(symbol),
  getStockRankings: () => stockApi.getStocks(),
  getUserStockAccess: () => Promise.resolve({ hasAccess: true }),
  getWatchlist: () => stockApi.getWatchlist(),
  addToWatchlist: (symbol: string, notes?: string) => stockApi.addToWatchlist(symbol, notes),
  removeFromWatchlist: (symbol: string) => stockApi.removeFromWatchlist(symbol),

  // Payment methods using new structure
  createPayment: (data: any) => paymentApi.createPayment(data),
  validatePayment: (paymentId: string) => paymentApi.verifyPayment(paymentId),
  getPaymentStatus: (paymentId: string) => paymentApi.getPaymentStatus(paymentId),
  getTransactionHistory: () => getTransactionHistory(),
  getPlanDetails: (id: string) => paymentApi.getPlan(id),
  initQRPayment: () => Promise.reject(new Error('QR Payment not implemented')),

  // Feature access methods using simple role system
  getUserFeatures: () => featureAccessApi.getUserFeatures(),
  checkFeatureAccess: (feature: string) => featureAccessApi.checkFeatureAccess(feature),
  checkPermission: (permission: string) => featureAccessApi.checkPermission(permission),
  checkRankingAccess: () => featureAccessApi.checkFeatureAccess('analytics'),
  
  // Deprecated methods that will throw errors
  get: () => { throw new Error('Use apiClient.domain.method() instead of generic get()'); },
  post: () => { throw new Error('Use apiClient.domain.method() instead of generic post()'); },
  put: () => { throw new Error('Use apiClient.domain.method() instead of generic put()'); },
  delete: () => { throw new Error('Use apiClient.domain.method() instead of generic delete()'); },
  
  getStocks: stockApi.getStocks,
  preloadStocks: async (_symbols: string[]) => {
    // This should be called from server context only
    throw new Error('preloadStocks must be called from server components or server actions');
  },
  checkStockCacheStatus: async (_symbols: string[]) => {
    // This should be called from server context only
    throw new Error('checkStockCacheStatus must be called from server components or server actions');
  },
};

// Export the new structure as the primary interface
export { apiClient };