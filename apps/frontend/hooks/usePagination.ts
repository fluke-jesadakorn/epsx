import { useState, useCallback } from 'react';

interface UsePaginationProps {
  initialPage?: number;
  initialLimit?: number;
  onPageChange?: (page: number, limit: number) => void;
}

export function usePagination({
  initialPage = 1,
  initialLimit = 10,
  onPageChange
}: UsePaginationProps = {}) {
  const [currentPage, setCurrentPage] = useState(initialPage);
  const [limit, setLimit] = useState(initialLimit);
  const [isLoading, setIsLoading] = useState(false);

  const handlePageChange = useCallback((newPage: number) => {
    setCurrentPage(newPage);
    onPageChange?.(newPage, limit);
  }, [limit, onPageChange]);

  const handleLimitChange = useCallback((newLimit: number) => {
    setLimit(newLimit);
    setCurrentPage(1); // Reset to first page when changing limit
    onPageChange?.(1, newLimit);
  }, [onPageChange]);

  const reset = useCallback(() => {
    setCurrentPage(initialPage);
    setLimit(initialLimit);
  }, [initialPage, initialLimit]);

  return {
    currentPage,
    limit,
    isLoading,
    setIsLoading,
    handlePageChange,
    handleLimitChange,
    reset
  };
}
