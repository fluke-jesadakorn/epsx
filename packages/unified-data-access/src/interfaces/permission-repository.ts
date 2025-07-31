import { BaseRepository, ListResult } from "./base-repository";

// Permission domain types
export interface Permission {
  id: string;
  name: string;
  resource: string;
  action: string;
  description?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface Role {
  id: string;
  name: string;
  description?: string;
  permissions: Permission[];
  isSystemRole: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface UserPermission {
  userId: string;
  permissionId: string;
  grantedAt: Date;
  grantedBy: string;
  expiresAt?: Date;
}

export interface PermissionProfile {
  id: string;
  name: string;
  description?: string;
  permissions: Permission[];
  isDefault: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface CreatePermissionInput {
  name: string;
  resource: string;
  action: string;
  description?: string;
}

export interface CreateRoleInput {
  name: string;
  description?: string;
  permissionIds: string[];
}

export interface PermissionFilters {
  resource?: string;
  action?: string;
  name?: string;
}

export interface RoleFilters {
  name?: string;
  isSystemRole?: boolean;
}

// Extended permission repository interface
export interface PermissionRepository extends BaseRepository<Permission, string, CreatePermissionInput, Partial<CreatePermissionInput>> {
  // Permission operations
  findByResource(resource: string): Promise<Permission[]>;
  findByAction(action: string): Promise<Permission[]>;
  checkPermissionExists(resource: string, action: string): Promise<boolean>;
  
  // Role operations
  getRoles(): Promise<Role[]>;
  getRole(id: string): Promise<Role | null>;
  createRole(data: CreateRoleInput): Promise<Role>;
  updateRole(id: string, data: Partial<CreateRoleInput>): Promise<Role>;
  deleteRole(id: string): Promise<void>;
  addPermissionToRole(roleId: string, permissionId: string): Promise<Role>;
  removePermissionFromRole(roleId: string, permissionId: string): Promise<Role>;
  
  // User permission operations
  getUserPermissions(userId: string): Promise<Permission[]>;
  getUserEffectivePermissions(userId: string): Promise<Permission[]>; // includes role permissions
  grantPermissionToUser(userId: string, permissionId: string, grantedBy: string, expiresAt?: Date): Promise<void>;
  revokePermissionFromUser(userId: string, permissionId: string): Promise<void>;
  
  // User role operations
  getUserRoles(userId: string): Promise<Role[]>;
  assignRoleToUser(userId: string, roleId: string): Promise<void>;
  removeRoleFromUser(userId: string, roleId: string): Promise<void>;
  
  // Permission checking
  userHasPermission(userId: string, resource: string, action: string): Promise<boolean>;
  userHasRole(userId: string, roleName: string): Promise<boolean>;
  
  // Permission profile operations
  getPermissionProfiles(): Promise<PermissionProfile[]>;
  getPermissionProfile(id: string): Promise<PermissionProfile | null>;
  createPermissionProfile(data: Omit<PermissionProfile, 'id' | 'createdAt' | 'updatedAt'>): Promise<PermissionProfile>;
  updatePermissionProfile(id: string, data: Partial<PermissionProfile>): Promise<PermissionProfile>;
  deletePermissionProfile(id: string): Promise<void>;
  assignPermissionProfile(userId: string, profileId: string, expiresAt?: Date): Promise<void>;
  revokePermissionProfile(userId: string, profileId: string): Promise<void>;
  
  // Filtered operations
  list(filters?: PermissionFilters): Promise<ListResult<Permission>>;
  listRoles(filters?: RoleFilters): Promise<ListResult<Role>>;
}