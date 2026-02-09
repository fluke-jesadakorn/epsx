import { useCallback, useEffect, useState } from 'react';

interface UseUsersOptions {
  searchTerm: string;
  statusFilter: string;
  packageFilter: string;
}

interface User {
  id: string;
  name: string;
  email: string;
  packageTier: string;
  status: 'active' | 'inactive' | 'suspended';
  lastActive: string;
  permissions: string[];
}

interface AdminUserStub {
  id: string;
  name?: string;
  email?: string;
  status: string;
  updated_at: string;
  permissions?: string[];
}

/**
 *
 * @param options
 */
export const useUsers = (options: UseUsersOptions) => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchUsers = useCallback(async () => {
    setLoading(true);
    try {
      // TODO: Implement with shared API client
      // Stub data for now
      const adminUsers: unknown[] = [];

      // Transform AdminUser to our User interface and apply filters
      let transformedUsers: User[] = (adminUsers as AdminUserStub[]).map(
        (user) => ({
          id: user.id,
          name: user.name ?? user.email ?? 'Unknown user',
          email: user.email ?? '',
          packageTier: 'user', // Map from roles/permissions as needed
          status: user.status as 'active' | 'inactive' | 'suspended',
          lastActive: new Date(user.updated_at).toLocaleDateString(),
          permissions: user.permissions ?? [], // Use structured permissions
        })
      );

      // Apply filters
      if (options.searchTerm) {
        const searchLower = options.searchTerm.toLowerCase();
        transformedUsers = transformedUsers.filter(
          user =>
            user.name.toLowerCase().includes(searchLower) ||
            user.email.toLowerCase().includes(searchLower)
        );
      }

      if (options.statusFilter && options.statusFilter !== 'all') {
        transformedUsers = transformedUsers.filter(
          user => user.status === options.statusFilter
        );
      }

      if (options.packageFilter && options.packageFilter !== 'all') {
        transformedUsers = transformedUsers.filter(
          user => user.packageTier === options.packageFilter
        );
      }

      setUsers(transformedUsers);
    } catch (_error) {

      console.error('Error fetching users', {
        error: _error instanceof Error ? _error.message : String(_error),
        searchTerm: options.searchTerm,
        statusFilter: options.statusFilter,
        packageFilter: options.packageFilter,
      });
      // Set empty array on error - no more mock data fallback
      setUsers([]);
    } finally {
      setLoading(false);
    }
  }, [options.searchTerm, options.statusFilter, options.packageFilter]);

  useEffect(() => {
    const debounceTimer = setTimeout(fetchUsers, 300);
    return () => clearTimeout(debounceTimer);
  }, [fetchUsers]);

  return { users, loading, refetch: fetchUsers };
};

export type { User, UseUsersOptions };
