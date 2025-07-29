export interface ApiResponse<T = any> {
  data?: T;
  error?: string;
  details?: string;
  message?: string;
}

export interface ApiError {
  error: string;
  details?: string;
  status?: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

export interface CountResponse {
  count: number;
}

export interface RequestConfig extends RequestInit {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  headers?: Record<string, string>;
  timeout?: number;
}