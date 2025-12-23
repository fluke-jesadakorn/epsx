/**
 * SHARED PERMISSION UTILITIES
 * Permission checking logic shared across admin-frontend and frontend
 */

/**
 * Check if user has a specific permission
 * Supports exact matches and wildcard patterns
 * @param userPermissions - Array of permission strings the user has
 * @param requiredPermission - The permission to check for
 * @returns true if permission is granted
 */
export function checkPermission(userPermissions: string[], requiredPermission: string): boolean {
    if (!userPermissions || userPermissions.length === 0) return false;

    return userPermissions.some(p => {
        // Exact match
        if (p === requiredPermission) return true;

        // Global wildcard
        if (p === '*' || p === '*:*:*') return true;

        // Platform/resource/action wildcard matching
        const [pPlatform, pResource, pAction] = p.split(':');
        const [rPlatform, rResource, rAction] = requiredPermission.split(':');

        if (pPlatform === rPlatform) {
            // Platform-wide wildcard (e.g., 'admin:*:*')
            if (pResource === '*' && pAction === '*') return true;
            // Resource-wide wildcard (e.g., 'admin:users:*')
            if (pResource === rResource && pAction === '*') return true;
        }

        return false;
    });
}

/**
 * Check if user has ANY of the specified permissions
 * @param userPermissions - Array of permission strings the user has
 * @param permissions - Array of permissions to check (OR logic)
 * @returns true if user has at least one of the permissions
 */
export function hasAnyPermission(userPermissions: string[], permissions: string[]): boolean {
    if (!permissions || permissions.length === 0) return true;
    return permissions.some(permission => checkPermission(userPermissions, permission));
}

/**
 * Check if user has ALL of the specified permissions
 * @param userPermissions - Array of permission strings the user has
 * @param permissions - Array of permissions to check (AND logic)
 * @returns true if user has all of the permissions
 */
export function hasAllPermissions(userPermissions: string[], permissions: string[]): boolean {
    if (!permissions || permissions.length === 0) return true;
    return permissions.every(permission => checkPermission(userPermissions, permission));
}

/**
 * Create a permission checker function bound to a user's permissions
 * Useful for repeated permission checks within a component
 * @param userPermissions - Array of permission strings the user has
 * @returns A function that checks a single permission
 */
export function createPermissionChecker(userPermissions: string[]): (permission: string) => boolean {
    return (permission: string) => checkPermission(userPermissions, permission);
}
