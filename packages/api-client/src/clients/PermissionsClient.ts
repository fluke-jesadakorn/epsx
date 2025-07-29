import { BaseHttpClient } from '../base/BaseHttpClient';
import type {
  Permission,
  Role,
  PermissionCheckRequest,
  PermissionCheckResponse,
  UserPermissionStatus,
  ApiResponse,
} from '@epsx/types';

export class PermissionsClient extends BaseHttpClient {
  async checkPermission(data: PermissionCheckRequest): Promise<ApiResponse<PermissionCheckResponse>> {
    return this.post<PermissionCheckResponse>('/api/permissions/check', data);
  }

  async getUserPermissions(userId: string): Promise<ApiResponse<UserPermissionStatus>> {
    return this.get<UserPermissionStatus>(`/api/permissions/users/${userId}`);
  }

  async getCurrentUserPermissions(): Promise<ApiResponse<UserPermissionStatus>> {
    return this.get<UserPermissionStatus>('/api/permissions/me');
  }

  async getAllPermissions(): Promise<ApiResponse<Permission[]>> {
    return this.get<Permission[]>('/api/permissions');
  }

  async getAllRoles(): Promise<ApiResponse<Role[]>> {
    return this.get<Role[]>('/api/permissions/roles');
  }

  async assignRoleToUser(userId: string, roleId: string): Promise<ApiResponse<void>> {
    return this.post<void>(`/api/permissions/users/${userId}/roles`, { roleId });
  }

  async removeRoleFromUser(userId: string, roleId: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/permissions/users/${userId}/roles/${roleId}`);
  }

  async grantPermissionToUser(userId: string, permissionId: string): Promise<ApiResponse<void>> {
    return this.post<void>(`/api/permissions/users/${userId}/permissions`, { permissionId });
  }

  async revokePermissionFromUser(userId: string, permissionId: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/permissions/users/${userId}/permissions/${permissionId}`);
  }

  // Batch operations
  async checkMultiplePermissions(requests: PermissionCheckRequest[]): Promise<ApiResponse<PermissionCheckResponse[]>> {
    return this.post<PermissionCheckResponse[]>('/api/permissions/check/batch', { requests });
  }

  async bulkAssignRoles(userIds: string[], roleIds: string[]): Promise<ApiResponse<void>> {
    return this.post<void>('/api/permissions/bulk/assign-roles', { userIds, roleIds });
  }

  async bulkRevokeRoles(userIds: string[], roleIds: string[]): Promise<ApiResponse<void>> {
    return this.post<void>('/api/permissions/bulk/revoke-roles', { userIds, roleIds });
  }
}