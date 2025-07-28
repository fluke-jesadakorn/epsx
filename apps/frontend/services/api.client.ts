// Refactored to use server actions from @epsx/server-actions
import {
  // Auth actions
  getCurrentUser,
  updateProfile,
  login,
  logout,
  
  // Stock actions
  getBatchStocks,
  getStockData,
  getStockRankings,
  getUserStockAccess,
  getWatchlist,
  addToWatchlist,
  removeFromWatchlist,
  
  // Payment actions
  createPayment,
  validatePayment,
  getPaymentStatus,
  getTransactionHistory,
  getPlanDetails,
  initQRPayment,
  
  // Permission actions
  getUserPermissions,
  checkPermission,
  checkFeatureAccess,
  checkRankingAccess
} from '@epsx/server-actions';

// Legacy wrapper class for backward compatibility
class ApiService {
  async get<T>(endpoint: string): Promise<T> {
    // This method is deprecated - migrate to specific server actions
    throw new Error('Direct API calls are deprecated. Use specific server actions instead.');
  }

  async post<T>(endpoint: string, data?: any): Promise<T> {
    // This method is deprecated - migrate to specific server actions
    throw new Error('Direct API calls are deprecated. Use specific server actions instead.');
  }

  async put<T>(endpoint: string, data?: any): Promise<T> {
    // This method is deprecated - migrate to specific server actions
    throw new Error('Direct API calls are deprecated. Use specific server actions instead.');
  }

  async delete<T>(endpoint: string): Promise<T> {
    // This method is deprecated - migrate to specific server actions
    throw new Error('Direct API calls are deprecated. Use specific server actions instead.');
  }
}

export const api = new ApiService();

// Specific API endpoints using server actions
export const stockApi = {
  getStocks: async () => {
    const rankings = await getStockRankings();
    return rankings;
  },
  
  getSymbols: async () => {
    // This would need to be implemented as a new server action
    throw new Error('getSymbols needs to be implemented as a server action');
  },
  
  getStock: async (symbol: string) => {
    return await getStockData(symbol);
  },
  
  searchStocks: async (query: string) => {
    // This would need to be implemented as a new server action
    throw new Error('searchStocks needs to be implemented as a server action');
  },
  
  getStockHistory: async (symbol: string, period = '1y') => {
    // This would need to be implemented as a new server action
    throw new Error('getStockHistory needs to be implemented as a server action');
  },
};

export const rankingApi = {
  getRankings: async () => {
    return await getStockRankings();
  },
  
  getUserRanking: async (userId: string) => {
    // This would need to be implemented as a new server action
    throw new Error('getUserRanking needs to be implemented as a server action');
  },
  
  updateRanking: async (data: any) => {
    // This would need to be implemented as a new server action
    throw new Error('updateRanking needs to be implemented as a server action');
  },
};

export const paymentApi = {
  createPayment: async (data: {
    amount: number;
    currency: string;
    description?: string;
    orderNo: string;
  }) => {
    return await createPayment(data);
  },
  
  getPaymentStatus: async (paymentId: string) => {
    // For individual payment status, we'll use the general payment status for now
    return await getPaymentStatus();
  },
  
  verifyPayment: async (paymentId: string) => {
    return await validatePayment(paymentId);
  },
  
  getCryptoDepositAddress: async () => {
    // This would need to be implemented as a new server action
    throw new Error('getCryptoDepositAddress needs to be implemented as a server action');
  },
  
  getQrCode: async (paymentId: string) => {
    // This would need to be implemented as a new server action
    throw new Error('getQrCode needs to be implemented as a server action');
  },
};

export const userApi = {
  getProfile: async () => {
    return await getCurrentUser();
  },
  
  updateProfile: async (data: {
    name?: string;
    email?: string;
    preferences?: any;
  }) => {
    return await updateProfile(data);
  },
  
  getUserData: async () => {
    return await getCurrentUser();
  },
  
  updateUserData: async (data: {
    name?: string;
    email?: string;
    preferences?: any;
  }) => {
    return await updateProfile(data);
  },
  
  getUserById: async (id: string) => {
    // This would need to be implemented as a new server action
    throw new Error('getUserById needs to be implemented as a server action');
  },
  
  listUsers: async () => {
    // This would need to be implemented as a new server action
    throw new Error('listUsers needs to be implemented as a server action');
  },
  
  deleteUser: async (id: string) => {
    // This would need to be implemented as a new server action
    throw new Error('deleteUser needs to be implemented as a server action');
  },
};

// Legacy export for backward compatibility - will throw errors to encourage migration
export const apiClient = {
  getCurrentUser,
  updateProfile,
  login,
  logout,
  getBatchStocks,
  getStockData,
  getStockRankings,
  getUserStockAccess,
  getWatchlist,
  addToWatchlist,
  removeFromWatchlist,
  createPayment,
  validatePayment,
  getPaymentStatus,
  getTransactionHistory,
  getPlanDetails,
  initQRPayment,
  getUserPermissions,
  checkPermission,
  checkFeatureAccess,
  checkRankingAccess,
  
  // Deprecated methods that will throw errors
  get: () => { throw new Error('Use specific server actions instead of generic get()'); },
  post: () => { throw new Error('Use specific server actions instead of generic post()'); },
  put: () => { throw new Error('Use specific server actions instead of generic put()'); },
  delete: () => { throw new Error('Use specific server actions instead of generic delete()'); },
  getStocks: stockApi.getStocks,
  preloadStocks: async (symbols: string[]) => {
    // Import preloadStocks from server actions
    const { preloadStocks } = await import('@epsx/server-actions');
    return await preloadStocks(symbols);
  },
  checkStockCacheStatus: async (symbols: string[]) => {
    // Import checkStockCacheStatus from server actions
    const { checkStockCacheStatus } = await import('@epsx/server-actions');
    return await checkStockCacheStatus(symbols);
  },
  register: async (data: { email: string; password: string; name?: string }) => {
    // This would need to be implemented as a new server action
    throw new Error('register needs to be implemented as a server action');
  },
};