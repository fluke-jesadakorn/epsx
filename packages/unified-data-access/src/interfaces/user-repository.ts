import { BaseRepository, ListResult } from "./base-repository";

// User domain types (these would typically come from @epsx/types)
export interface User {
  id: string;
  email: string;
  firstName?: string;
  lastName?: string;
  role: string;
  status: 'active' | 'inactive' | 'suspended';
  createdAt: Date;
  updatedAt: Date;
  lastLoginAt?: Date;
}

export interface CreateUserInput {
  email: string;
  firstName?: string;
  lastName?: string;
  password: string;
  role?: string;
}

export interface UpdateUserInput {
  firstName?: string;
  lastName?: string;
  role?: string;
  status?: 'active' | 'inactive' | 'suspended';
}

export interface UserFilters {
  role?: string;
  status?: 'active' | 'inactive' | 'suspended';
  email?: string;
  createdAfter?: Date;
  createdBefore?: Date;
}

// Extended user repository interface with domain-specific operations
export interface UserRepository extends BaseRepository<User, string, CreateUserInput, UpdateUserInput> {
  // Authentication-specific operations
  findByEmail(email: string): Promise<User | null>;
  findByCredentials(email: string, password: string): Promise<User | null>;
  updatePassword(id: string, newPassword: string): Promise<void>;
  updateLastLogin(id: string): Promise<void>;
  
  // Role and permission operations
  findByRole(role: string): Promise<User[]>;
  updateUserRole(id: string, role: string): Promise<User>;
  
  // Status management
  activateUser(id: string): Promise<User>;
  deactivateUser(id: string): Promise<User>;
  suspendUser(id: string, reason?: string): Promise<User>;
  
  // Filtered list operations
  list(filters?: UserFilters): Promise<ListResult<User>>;
  
  // Bulk operations for admin
  bulkUpdateRole(userIds: string[], role: string): Promise<User[]>;
  bulkUpdateStatus(userIds: string[], status: 'active' | 'inactive' | 'suspended'): Promise<User[]>;
}