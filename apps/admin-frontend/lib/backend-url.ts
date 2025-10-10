/**
 * Get backend URL for API calls
 * All admin frontend calls go directly to backend - no API routes
 */
export function getBackendUrl(path: string = ''): string {
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  return path ? `${backendUrl}${path}` : backendUrl;
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
