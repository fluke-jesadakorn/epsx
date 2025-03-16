import { Paginate } from '@epsx/types';

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
