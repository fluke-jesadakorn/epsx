import { Paginate } from '../types/pagination';

export function formatPaginationResponse<T>(
  data: T[],
  total: number,
  page: number,
  limit: number
): Paginate<T> {
  return {
    data,
    total,
    page,
    limit
  };
}
