import { Injectable } from '@nestjs/common';

export interface PaginationParams {
  limit?: number;
  skip?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
}

@Injectable()
export class PaginationService {
  getPaginationParams(params: PaginationParams) {
    return {
      limit: params.limit ? parseInt(String(params.limit), 10) : 20,
      skip: params.skip ? parseInt(String(params.skip), 10) : 0,
      sortBy: params.sortBy || 'createdAt',
      sortOrder: params.sortOrder || 'desc'
    };
  }

  createPaginatedResponse<T>(
    data: T[],
    total: number,
    params: PaginationParams
  ): PaginatedResponse<T> {
    const { limit, skip } = this.getPaginationParams(params);
    const page = Math.floor(skip / limit) + 1;

    return {
      data,
      total,
      page,
      limit
    };
  }
}
