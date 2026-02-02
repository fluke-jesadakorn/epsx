/**
 * Get backend URL for API calls
 * All admin frontend calls go directly to backend - no API routes
 * @param path
 */
export function getBackendUrl(path = ''): string {
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://127.0.0.1:8080';

  if (!path) { return backendUrl; }

  return `${backendUrl}${path}`;
}

/**
 * Fetch from backend directly with credentials
 * @param path
 * @param options
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
