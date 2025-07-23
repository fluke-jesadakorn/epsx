import { useCallback, useEffect, useState } from 'react';
import { iamService } from '../../services/iamService';
import type { UserWithPermissions } from '../../types/admin/iam-enhanced';

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

export const useUsers = (options: UseUsersOptions) => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchUsers = useCallback(async () => {
    setLoading(true);
    try {
      // Get users from existing IAM service
      const iamUsers = await iamService.getUsers();

      // Transform to our User interface and apply filters
      let transformedUsers: User[] = iamUsers.map(
        (user: UserWithPermissions) => ({
          id: user.id,
          name: user.displayName || user.name || user.email || 'Unknown User',
          email: user.email || '',
          packageTier: user.packageTier || 'free',
          status: user.subscriptionStatus === 'active' ? 'active' : 'inactive',
          lastActive: user.lastActivity
            ? new Date(user.lastActivity).toLocaleDateString()
            : 'Never',
          permissions: user.effectivePermissions?.map((p) => p.featureId) || [],
        }),
      );

      // Apply filters
      if (options.searchTerm) {
        const searchLower = options.searchTerm.toLowerCase();
        transformedUsers = transformedUsers.filter(
          (user) =>
            user.name.toLowerCase().includes(searchLower) ||
            user.email.toLowerCase().includes(searchLower),
        );
      }

      if (options.statusFilter && options.statusFilter !== 'all') {
        transformedUsers = transformedUsers.filter(
          (user) => user.status === options.statusFilter,
        );
      }

      if (options.packageFilter && options.packageFilter !== 'all') {
        transformedUsers = transformedUsers.filter(
          (user) => user.packageTier === options.packageFilter,
        );
      }

      setUsers(transformedUsers);
    } catch (error) {
      console.error('Error fetching users:', error);
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
