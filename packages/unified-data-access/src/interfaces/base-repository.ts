import { z } from "zod";

// Base repository operations that all repositories should support
export interface BaseRepository<T, ID = string, CreateInput = Partial<T>, UpdateInput = Partial<T>> {
  // Single entity operations
  get(id: ID): Promise<T | null>;
  getRequired(id: ID): Promise<T>;
  
  // Multiple entity operations  
  list(filters?: Record<string, any>, options?: ListOptions): Promise<ListResult<T>>;
  search(query: string, options?: SearchOptions): Promise<ListResult<T>>;
  
  // Mutation operations
  create(data: CreateInput): Promise<T>;
  update(id: ID, data: UpdateInput): Promise<T>;
  delete(id: ID): Promise<void>;
  
  // Batch operations
  bulkCreate(data: CreateInput[]): Promise<T[]>;
  bulkUpdate(updates: Array<{ id: ID; data: UpdateInput }>): Promise<T[]>;
  bulkDelete(ids: ID[]): Promise<void>;
}

// Options for list operations
export interface ListOptions {
  page?: number;
  limit?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  include?: string[];
}

// Options for search operations
export interface SearchOptions extends ListOptions {
  fields?: string[];
}

// Standard list result format
export interface ListResult<T> {
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

// Repository error types
export class RepositoryError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: any
  ) {
    super(message);
    this.name = 'RepositoryError';
  }
}

export class NotFoundError extends RepositoryError {
  constructor(resource: string, id: string) {
    super(`${resource} with id ${id} not found`, 'NOT_FOUND', { resource, id });
    this.name = 'NotFoundError';
  }
}

export class ValidationError extends RepositoryError {
  constructor(message: string, validationDetails: any) {
    super(message, 'VALIDATION_ERROR', validationDetails);
    this.name = 'ValidationError';
  }
}

// Environment context for repositories
export type RepositoryContext = 'server' | 'client';

// Repository factory interface
export interface RepositoryFactory {
  getUserRepository(): BaseRepository<any>;
  getPaymentRepository(): BaseRepository<any>;
  getPermissionRepository(): BaseRepository<any>;
  getAnalyticsRepository(): BaseRepository<any>;
}