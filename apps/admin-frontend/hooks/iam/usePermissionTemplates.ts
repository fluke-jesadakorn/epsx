import { useCallback, useEffect, useState } from 'react';

interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  permissions: string[];
  createdAt: string;
  lastModified: string;
  usageCount: number;
  isActive: boolean;
}

interface UsePermissionTemplatesOptions {
  searchTerm?: string;
  categoryFilter?: string;
}

const usePermissionTemplates = (options: UsePermissionTemplatesOptions) => {
  const [templates, setTemplates] = useState<PermissionTemplate[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchTemplates = useCallback(async () => {
    setLoading(true);
    try {
      // Mock data for now - in real implementation this would fetch from API
      let mockTemplates: PermissionTemplate[] = [
        {
          id: '1',
          name: 'Basic User',
          description: 'Standard permissions for regular users',
          category: 'User',
          permissions: ['read:users', 'read:analytics'],
          createdAt: '2024-01-01T00:00:00Z',
          lastModified: '2024-01-01T00:00:00Z',
          usageCount: 150,
          isActive: true,
        },
        {
          id: '2',
          name: 'Premium User',
          description: 'Enhanced permissions for premium subscribers',
          category: 'User',
          permissions: [
            'read:users',
            'read:analytics',
            'read:premium-features',
          ],
          createdAt: '2024-01-02T00:00:00Z',
          lastModified: '2024-01-02T00:00:00Z',
          usageCount: 75,
          isActive: true,
        },
        {
          id: '3',
          name: 'Admin',
          description: 'Full administrative permissions',
          category: 'Admin',
          permissions: [
            'read:users',
            'write:users',
            'delete:users',
            'admin:all',
          ],
          createdAt: '2024-01-03T00:00:00Z',
          lastModified: '2024-01-03T00:00:00Z',
          usageCount: 25,
          isActive: true,
        },
        {
          id: '4',
          name: 'Support Agent',
          description: 'Customer support permissions',
          category: 'Support',
          permissions: ['read:users', 'support:tickets', 'support:chat'],
          createdAt: '2024-01-04T00:00:00Z',
          lastModified: '2024-01-04T00:00:00Z',
          usageCount: 45,
          isActive: true,
        },
        {
          id: '5',
          name: 'Manager',
          description: 'Team management permissions',
          category: 'Manager',
          permissions: [
            'read:users',
            'write:users',
            'read:analytics',
            'manage:team',
          ],
          createdAt: '2024-01-05T00:00:00Z',
          lastModified: '2024-01-05T00:00:00Z',
          usageCount: 30,
          isActive: true,
        },
      ];

      // Apply filters
      if (options.searchTerm) {
        const searchLower = options.searchTerm.toLowerCase();
        mockTemplates = mockTemplates.filter(
          (template) =>
            template.name.toLowerCase().includes(searchLower) ||
            template.description.toLowerCase().includes(searchLower),
        );
      }

      if (options.categoryFilter && options.categoryFilter !== 'all') {
        mockTemplates = mockTemplates.filter(
          (template) => template.category === options.categoryFilter,
        );
      }

      setTemplates(mockTemplates);
    } catch (error) {
      console.error('Error fetching permission templates:', error);
      setTemplates([]);
    } finally {
      setLoading(false);
    }
  }, [options.searchTerm, options.categoryFilter]);

  useEffect(() => {
    fetchTemplates();
  }, [fetchTemplates]);

  return { templates, loading, refetch: fetchTemplates };
};

export { usePermissionTemplates };
export type { PermissionTemplate, UsePermissionTemplatesOptions };
