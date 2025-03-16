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
