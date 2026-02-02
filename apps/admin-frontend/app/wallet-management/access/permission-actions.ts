'use server';

import { CreatePermissionRequest, PermissionDefinition, permissionsClient } from '@/lib/api/permissions-client';
import { getAdminServerActionClient } from '@/shared/utils/server-fetch';
import { revalidatePath } from 'next/cache';

export interface ActionResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
}

export async function getPermissionsAction(): Promise<ActionResponse<PermissionDefinition[]>> {
    try {
        const client = await getAdminServerActionClient();
        const permissions = await permissionsClient.listPermissions(client);
        return { success: true, data: permissions };
    } catch (error) {
        console.error('Failed to fetch permissions:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Failed to fetch permissions'
        };
    }
}

export async function createPermissionAction(data: CreatePermissionRequest): Promise<ActionResponse<PermissionDefinition>> {
    try {
        const client = await getAdminServerActionClient();
        const permission = await permissionsClient.createPermission(data, client);
        revalidatePath('/wallet-management/access');
        return { success: true, data: permission };
    } catch (error) {
        console.error('Failed to create permission:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Failed to create permission'
        };
    }
}

export async function deletePermissionAction(id: string): Promise<ActionResponse<void>> {
    try {
        const client = await getAdminServerActionClient();
        await permissionsClient.deletePermission(id, client);
        revalidatePath('/wallet-management/access');
        return { success: true };
    } catch (error) {
        console.error('Failed to delete permission:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Failed to delete permission'
        };
    }
}
