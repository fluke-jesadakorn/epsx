// Common interfaces
export interface PaginationParams {
  limit: number;
  skip: number;
}

export interface PaginationMetadata {
  total: number;
  page: number;
  limit: number;
  totalPages: number;
  skip?: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  metadata: PaginationMetadata;
}
