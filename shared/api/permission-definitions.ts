/**
 * PERMISSION DEFINITIONS API CLIENT
 *
 * Provides API client for fetching permission definitions with human-readable
 * titles and notes for display in the UI.
 */

import type { ApiResponse, UnifiedApiClient } from '../utils/api-client';
import { logger } from '../utils/logger';

// ============================================================================
// PERMISSION DEFINITION TYPES
// ============================================================================

export interface PermissionDefinition {
    id: string;
    permission_string: string;  // Technical code: "epsx:analytics:view"
    name: string | null;        // Human-readable title: "View Analytics"
    description: string | null; // Human-readable note: "View basic analytics..."
    platform: string;           // Platform: "epsx", "admin", etc.
    category: string | null;    // Category: "analytics", "trading", etc.
    is_system: boolean;
    is_active: boolean;
    created_at: string;
}

// ============================================================================
// PERMISSION DEFINITIONS API CLIENT
// ============================================================================

/**
 * Create permission definitions API client
 */
export function createPermissionDefinitionsApiClient(apiClient: UnifiedApiClient) {
    return {
        /**
         * Get all permission definitions with human-readable titles and notes
         */
        async listDefinitions(): Promise<ApiResponse<PermissionDefinition[]>> {
            return apiClient.get<PermissionDefinition[]>('/api/permissions/definitions');
        },
    };
}

// ============================================================================
// PERMISSION DISPLAY UTILITIES
// ============================================================================

// Cache for permission definitions (client-side)
let permissionDefinitionsCache: Map<string, PermissionDefinition> | null = null;
let cacheLoadPromise: Promise<void> | null = null;

/**
 * Load and cache permission definitions
 */
export async function loadPermissionDefinitions(apiClient: UnifiedApiClient): Promise<Map<string, PermissionDefinition>> {
    if (permissionDefinitionsCache) {
        return permissionDefinitionsCache;
    }

    // Avoid multiple concurrent loads
    if (cacheLoadPromise) {
        await cacheLoadPromise;
        const cached = permissionDefinitionsCache as Map<string, PermissionDefinition> | null;
        if (cached) {
            return cached;
        }
        return new Map();
    }

    cacheLoadPromise = (async () => {
        try {
            const client = createPermissionDefinitionsApiClient(apiClient);
            const response = await client.listDefinitions();

            // The API client wraps the backend response, so we have:
            // response.success = true (from api-client)
            // response.data = { success: true, data: [...] } (from backend)
            // We need to handle both cases where data could be the array directly
            // or nested inside another object
            if (response.success && response.data) {
                permissionDefinitionsCache = new Map();

                // Check if response.data is the array directly or nested
                const definitions: PermissionDefinition[] = Array.isArray(response.data)
                    ? response.data
                    : (response.data as { data?: PermissionDefinition[] }).data ?? [];

                for (const def of definitions) {
                    cacheDefinition(def);
                }
            } else {
                permissionDefinitionsCache = new Map();
            }
        } catch (error) {
            logger.error('Failed to load permission definitions:', error);
            permissionDefinitionsCache = new Map();
        }
    })();

    await cacheLoadPromise;
    const finalCache = permissionDefinitionsCache as Map<string, PermissionDefinition> | null;
    return finalCache ?? new Map();
}

/**
 * Helper to cache a single definition
 */
function cacheDefinition(def: PermissionDefinition): void {
    if (permissionDefinitionsCache !== null && typeof def.permission_string === 'string' && def.permission_string !== '') {
        permissionDefinitionsCache.set(def.permission_string, def);
    }
}

/**
 * Get human-readable title for a permission string
 * Falls back to formatted permission string if no definition found
 */
export function getPermissionTitle(
    permission: string,
    definitions: Map<string, PermissionDefinition>
): string {
    const def = definitions.get(permission);
    if (def !== undefined && typeof def.name === 'string' && def.name !== '') {
        return def.name;
    }

    // Fallback: Format the permission string nicely
    // "epsx:analytics:view" -> "Analytics View"
    const parts = permission.split(':');
    if (parts.length >= 3) {
        const resource = parts[1] ?? '';
        const action = parts[2] ?? '';
        return `${capitalize(resource)} ${capitalize(action)}`;
    }
    return permission;
}

/**
 * Get human-readable description/note for a permission string
 */
export function getPermissionNote(
    permission: string,
    definitions: Map<string, PermissionDefinition>
): string | null {
    const def = definitions.get(permission);
    return def?.description ?? null;
}

/**
 * Get platform and category for a permission string
 */
export function getPermissionMeta(
    permission: string,
    definitions: Map<string, PermissionDefinition>
): { platform: string; category: string | null } {
    const def = definitions.get(permission);
    if (def) {
        return { platform: def.platform, category: def.category };
    }

    // Fallback: Parse from permission string
    const parts = permission.split(':');
    return {
        platform: parts[0] || 'unknown',
        category: parts[1] || null,
    };
}

/**
 * Clear the permission definitions cache (for testing or refresh)
 */
export function clearPermissionDefinitionsCache(): void {
    permissionDefinitionsCache = null;
    cacheLoadPromise = null;
}

// Helper function
function capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1).replace(/[-_]/g, ' ');
}
