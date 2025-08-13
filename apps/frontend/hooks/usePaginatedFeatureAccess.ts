'use client';

/**
 * Simple replacement for pagination feature access control
 * Provides basic pagination limits without complex tier restrictions
 */
export default function usePaginatedFeatureAccess() {
  // Basic tier information (simplified)
  const userTier = 'basic'; // Default to basic tier
  
  // Get maximum allowed limit based on user tier
  const getMaxAllowedLimit = () => {
    // Simplified logic - allow reasonable pagination limits
    return 50; // Basic limit for all users
  };

  // Check if user can access a specific page
  const canAccessPage = (page: number) => {
    // Simplified logic - allow access to reasonable page numbers
    return page <= 100; // Limit to first 100 pages for performance
  };

  // Get available page sizes based on user tier
  const getAvailablePageSizes = () => {
    // Simplified logic - basic page sizes for all users
    return [5, 10, 20, 50];
  };

  return {
    getMaxAllowedLimit,
    canAccessPage,
    getAvailablePageSizes,
    userTier,
  };
}