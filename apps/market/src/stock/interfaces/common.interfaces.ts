export interface PaginationParams {
  page?: number;
  limit?: number;
  skip?: number;
}

export interface StockScreenerResponse {
  data: {
    data: Array<{
      s: string; // symbol
      n: string; // name
    }>;
    resultsCount?: number;
  };
}

export function formatPaginationResponse<T>(
  data: T[],
  total: number,
  skip: number,
  limit: number
) {
  return {
    data,
    meta: {
      total,
      skip,
      limit,
      page: Math.floor(skip / limit) + 1,
      pages: Math.ceil(total / limit),
    },
  };
}

// Decorator implementation
export function Paginate() {
  return function (
    _target: any,
    _propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    return descriptor;
  };
}
