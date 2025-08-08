import { useState, useEffect as _useEffect } from 'react';
import { useAuth } from './useAuth';

interface PaginatedFeatureAccessOptions {
  feature: string;
  pageSize?: number;
}

/**
 * Hook for managing paginated feature access with parameters
 */
function usePaginatedFeatureAccessWithParams({ 
  feature, 
  pageSize = 10 
}: PaginatedFeatureAccessOptions) {
  const { user, loading } = useAuth();
  const [currentPage, setCurrentPage] = useState(1);
  
  const hasFeatureAccess = user?.permissions?.includes(`${feature}:read`) || false;
  
  const totalPages = hasFeatureAccess ? Math.ceil(100 / pageSize) : 0; // Default assumption
  
  const goToPage = (page: number) => {
    if (page >= 1 && page <= totalPages) {
      setCurrentPage(page);
    }
  };

  const nextPage = () => goToPage(currentPage + 1);
  const prevPage = () => goToPage(currentPage - 1);
  
  return {
    hasAccess: hasFeatureAccess,
    loading,
    currentPage,
    totalPages,
    pageSize,
    goToPage,
    nextPage,
    prevPage,
    canGoNext: currentPage < totalPages,
    canGoPrev: currentPage > 1
  };
}

export { usePaginatedFeatureAccessWithParams as usePaginatedFeatureAccess };

/**
 * Hook for managing paginated feature access without parameters (for analytics dashboard)
 */
export default function usePaginatedFeatureAccess() {
  const { user, loading } = useAuth();
  
  // Determine user tier based on role and subscription
  const getUserTier = () => {
    if (user?.role === 'system_administrator' || user?.role === 'admin') {
      return 'PLATINUM'; // Give SuperAdmin users full access
    }
    
    const subscriptionTier = user?.subscription_tier?.toLowerCase() || 'free';
    if (['premium', 'enterprise', 'platinum', 'gold'].includes(subscriptionTier)) {
      return 'GOLD';
    }
    if (subscriptionTier === 'silver') {
      return 'SILVER';
    }
    return 'BASIC';
  };

  const userTier = getUserTier();
  
  // Define limits based on user tier
  const getTierLimits = () => {
    switch (userTier) {
      case 'PLATINUM':
        return { maxLimit: 1000, pageSizes: [10, 25, 50, 100], maxPages: 100 };
      case 'GOLD':
        return { maxLimit: 500, pageSizes: [10, 25, 50], maxPages: 50 };
      case 'SILVER':
        return { maxLimit: 100, pageSizes: [10, 25], maxPages: 10 };
      case 'BASIC':
      default:
        return { maxLimit: 25, pageSizes: [10, 25], maxPages: 3 };
    }
  };

  const tierLimits = getTierLimits();

  const getMaxAllowedLimit = () => tierLimits.maxLimit;
  
  const getAvailablePageSizes = () => tierLimits.pageSizes;
  
  const canAccessPage = (page: number, limit: number) => {
    const totalItems = page * limit;
    return totalItems <= tierLimits.maxLimit;
  };
  
  return {
    getMaxAllowedLimit,
    canAccessPage,
    getAvailablePageSizes,
    userTier,
    loading
  };
}