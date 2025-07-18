import { useFeatureAccess } from '@/hooks/useFeatureAccess';

export function usePaginatedFeatureAccess() {
  const { canAccessRankings, userTier } = useFeatureAccess();

  const getMaxAllowedLimit = () => {
    // Basic users can only see 5 items max
    if (userTier === 'BRONZE') return 5;
    if (userTier === 'SILVER') return 25;
    if (userTier === 'GOLD') return 50;
    return 100; // PLATINUM
  };

  const canAccessPage = (page: number, limit: number) => {
    const maxItems = getMaxAllowedLimit();
    const requestedItems = page * limit;
    return requestedItems <= maxItems;
  };

  const getAvailablePageSizes = () => {
    const maxLimit = getMaxAllowedLimit();
    const pageSizes = [5, 10, 20, 50, 100];
    return pageSizes.filter(size => size <= maxLimit);
  };

  return {
    getMaxAllowedLimit,
    canAccessPage,
    canAccessRankings,
    getAvailablePageSizes,
    userTier
  };
}
