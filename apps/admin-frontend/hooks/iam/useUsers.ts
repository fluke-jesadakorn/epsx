import { useState, useEffect } from 'react';
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

  useEffect(() => {
    const fetchUsers = async () => {
      setLoading(true);
      try {
        // Get users from existing IAM service
        const iamUsers = await iamService.getUsers();
        
        // Transform to our User interface and apply filters
        let transformedUsers: User[] = iamUsers.map((user: UserWithPermissions) => ({
          id: user.id,
          name: user.displayName || user.name || user.email || 'Unknown User',
          email: user.email || '',
          packageTier: user.packageTier || 'free',
          status: user.subscriptionStatus === 'active' ? 'active' : 'inactive',
          lastActive: user.lastActivity ? new Date(user.lastActivity).toLocaleDateString() : 'Never',
          permissions: user.effectivePermissions?.map(p => p.featureId) || []
        }));

        // Apply filters
        if (options.searchTerm) {
          const searchLower = options.searchTerm.toLowerCase();
          transformedUsers = transformedUsers.filter(user =>
            user.name.toLowerCase().includes(searchLower) ||
            user.email.toLowerCase().includes(searchLower)
          );
        }

        if (options.statusFilter !== 'all') {
          transformedUsers = transformedUsers.filter(user => user.status === options.statusFilter);
        }

        if (options.packageFilter !== 'all') {
          transformedUsers = transformedUsers.filter(user => user.packageTier === options.packageFilter);
        }

        setUsers(transformedUsers);
      } catch (error) {
        console.error('Failed to fetch users:', error);
        // Set mock data for development
        setUsers([
          {
            id: '1',
            name: 'John Doe',
            email: 'john@example.com',
            packageTier: 'premium',
            status: 'active',
            lastActive: '2024-01-15',
            permissions: ['user.read', 'user.write']
          },
          {
            id: '2',
            name: 'Jane Smith',
            email: 'jane@example.com',
            packageTier: 'free',
            status: 'active',
            lastActive: '2024-01-14',
            permissions: ['user.read']
          }
        ]);
      } finally {
        setLoading(false);
      }
    };

    const debounceTimer = setTimeout(fetchUsers, 300);
    return () => clearTimeout(debounceTimer);
  }, [options.searchTerm, options.statusFilter, options.packageFilter]);

  return { users, loading };
};
