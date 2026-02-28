import { QueryClient } from '@tanstack/react-query';

export type Platform = 'admin' | 'frontend';

/**
 * Create a QueryClient with platform-specific defaults.
 *
 * Admin: Higher retry count, shorter cache time (data changes frequently)
 * Frontend: Lower retry count, infinite cache time (more static)
 */
export function createQueryClient(platform: Platform): QueryClient {
  const isAdmin = platform === 'admin';

  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: isAdmin ? 3 : 1,
        staleTime: isAdmin ? 30_000 : 60_000,
        gcTime: isAdmin ? 300000 : Infinity, // 5 minutes for admin, infinite for frontend
        refetchOnWindowFocus: false,
      },
    },
  });
}
