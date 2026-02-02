import { createAdminApiClient, handleSimpleRequest, UnifiedApiClient } from '@/shared/utils/api-client';

export interface PermissionDefinition {
    id: string;
    permission_string: string;
    name: string | null;
    description: string | null;
    platform: string;
    category: string | null;
    is_system: boolean;
    is_active: boolean;
    created_at: string;
}

export interface CreatePermissionRequest {
    permission: string;
    name?: string;
    description?: string;
    platform?: string;
    category?: string;
}

export const permissionsClient = {
    /**
     * List all available permission definitions
     */
    async listPermissions(client?: UnifiedApiClient): Promise<PermissionDefinition[]> {
        const apiClient = client || createAdminApiClient();
        return handleSimpleRequest<PermissionDefinition[]>(
            apiClient,
            'get',
            '/api/permissions/definitions'
        );
    },

    /**
     * Create a new permission definition
     */
    async createPermission(data: CreatePermissionRequest, client?: UnifiedApiClient): Promise<PermissionDefinition> {
        const apiClient = client || createAdminApiClient();
        return handleSimpleRequest<PermissionDefinition>(
            apiClient,
            'post',
            '/api/permissions/definitions',
            data
        );
    },

    /**
     * Delete a permission definition
     */
    async deletePermission(id: string, client?: UnifiedApiClient): Promise<void> {
        const apiClient = client || createAdminApiClient();
        return handleSimpleRequest<void>(
            apiClient,
            'delete',
            `/api/permissions/definitions/${id}`
        );
    }
};
