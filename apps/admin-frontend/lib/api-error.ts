import { redirect } from 'next/navigation';

/** Redirect to /access-denied if response is 403 */
export function redirectOnForbidden(
  res: { success: boolean; error?: { status?: number; message?: string } | null },
  route: string
): void {
  if (!res.success && res.error?.status === 403) {
    const params = new URLSearchParams({
      route: encodeURIComponent(route),
      context: 'admin',
      permission: res.error.message ?? 'Unknown permission',
    });
    redirect(`/access-denied?${params.toString()}`);
  }
}

/** Re-throw Next.js redirect errors (have `digest` property) */
export function rethrowRedirect(e: unknown): void {
  if (typeof e === 'object' && e !== null && 'digest' in e) throw e;
}
