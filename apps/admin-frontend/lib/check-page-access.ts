import { redirect } from 'next/navigation';

import { getServerSession } from '@/lib/auth/server';

/**
 * Permission wildcard matching (mirrors backend core/permissions.rs).
 * Supports: exact, `*:*` super-admin, `platform:*:*`, `platform:resource:*`
 */
function hasPermission(userPerms: string[], required: string): boolean {
  return userPerms.some(p => {
    if (p === required || p === '*:*') { return true; }
    const parts = required.split(':');
    if (parts.length >= 3) {
      if (p === `${parts[0]}:*:*`) { return true; }
      if (p === `${parts[0]}:${parts[1]}:*`) { return true; }
    }
    return false;
  });
}

/**
 * Server-side permission gate.
 * Reads JWT from cookies, checks permission, redirects to /access-denied if denied.
 */
export async function checkPageAccess(requiredPermission: string, route: string): Promise<void> {
  try {
    const session = await getServerSession();
    if (!session?.user) { return; } // Not authenticated — auth middleware handles redirect

    const perms = session.user.permissions;
    if (hasPermission(perms, requiredPermission)) { return; } // Access granted

    const params = new URLSearchParams({
      route: encodeURIComponent(route),
      context: 'admin',
      permission: requiredPermission,
    });
    redirect(`/access-denied?${params.toString()}`);
  } catch (e: unknown) {
    if (typeof e === 'object' && e !== null && 'digest' in e) { throw e; }
  }
}
