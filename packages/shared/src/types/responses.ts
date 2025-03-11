
// Pagination types
export interface PaginationParams {
  page?: number;
  limit?: number;
}

export interface Paginate<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
}

export interface PaginateResult<T> {
  data: T[];
  total: number;
  currentPage: number;
  totalPages: number;
  hasNextPage: boolean;
  hasPreviousPage: boolean;
}

// Utility functions
export function formatPaginationResponse<T>(
  data: T[],
  total: number,
  skip: number,
  limit: number
): Paginate<T> {
  const page = Math.floor(skip / limit) + 1;

  return {
    data,
    total,
    page,
    limit
  };
}
