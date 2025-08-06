export interface Pagination {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface SortOptions {
  field: string;
  direction: 'asc' | 'desc';
}

export interface FilterOptions {
  field: string;
  operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'in' | 'like';
  value: unknown;
}

export interface SearchOptions {
  query: string;
  fields?: string[];
  exact?: boolean;
}

export interface ListOptions {
  pagination?: Partial<Pagination>;
  sort?: SortOptions[];
  filters?: FilterOptions[];
  search?: SearchOptions;
}

export interface TimestampFields {
  createdAt: Date;
  updatedAt: Date;
}

export interface AuditFields extends TimestampFields {
  createdBy: string;
  updatedBy: string;
}

export type EntityStatus = 'active' | 'inactive' | 'pending' | 'archived' | 'deleted';

export interface BaseEntity {
  id: string;
  status: EntityStatus;
}

export interface AuditableEntity extends BaseEntity, AuditFields {}

export type Environment = 'development' | 'staging' | 'production' | 'test';

export interface AppConfig {
  environment: Environment;
  apiUrl: string;
  version: string;
  features: Record<string, boolean>;
}