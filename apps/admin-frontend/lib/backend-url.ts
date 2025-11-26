/**
 * Get backend URL for API calls
 * All admin frontend calls go directly to backend - no API routes
 * Automatically adds /api/v1 prefix if missing for standardized API routes
 */
export function getBackendUrl(path: string = ''): string {
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';

  if (!path) return backendUrl;

  // If path starts with /api but doesn't have /api/v1, add it
  if (path.startsWith('/api/') && !path.startsWith('/api/v1/')) {
    const normalizedPath = path.replace(/^\/api\//, '/api/v1/');
    return `${backendUrl}${normalizedPath}`;
  }

  return `${backendUrl}${path}`;
}

/**
 * Fetch from backend directly with credentials
 */
export async function fetchBackend(path: string, options?: RequestInit): Promise<Response> {
  return fetch(getBackendUrl(path), {
    credentials: 'include',
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });
}
