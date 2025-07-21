import { useState, useEffect } from 'react';

interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  category: 'User' | 'Admin' | 'Support' | 'Manager';
  permissions: string[];
  usageCount: number;
  isActive: boolean;
}

interface UsePermissionTemplatesOptions {
  searchTerm: string;
  categoryFilter: string;
}

export const usePermissionTemplates = (options: UsePermissionTemplatesOptions) => {
  const [templates, setTemplates] = useState<PermissionTemplate[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchTemplates = async () => {
      setLoading(true);
      try {
        // Mock data for now - in real implementation this would fetch from API
        let mockTemplates: PermissionTemplate[] = [
          {
            id: '1',
            name: 'Basic User',
            description: 'Standard permissions for regular users',
            category: 'User',
            permissions: ['user.read', 'user.profile.edit'],
            usageCount: 150,
            isActive: true
          },
          {
            id: '2',
            name: 'Premium User',
            description: 'Enhanced permissions for premium subscribers',
            category: 'User',
            permissions: ['user.read', 'user.profile.edit', 'premium.features.access'],
            usageCount: 75,
            isActive: true
          },
          {
            id: '3',
            name: 'Admin',
            description: 'Full administrative permissions',
            category: 'Admin',
            permissions: ['admin.users.manage', 'admin.permissions.manage', 'admin.system.config'],
            usageCount: 5,
            isActive: true
          },
          {
            id: '4',
            name: 'Support Staff',
            description: 'Customer support team permissions',
            category: 'Support',
            permissions: ['support.tickets.read', 'support.tickets.write', 'user.read'],
            usageCount: 12,
            isActive: true
          },
          {
            id: '5',
            name: 'Manager',
            description: 'Team management permissions',
            category: 'Manager',
            permissions: ['team.read', 'team.manage', 'reports.access'],
            usageCount: 8,
            isActive: true
          }
        ];

        // Apply filters
        if (options.searchTerm) {
          const searchLower = options.searchTerm.toLowerCase();
          mockTemplates = mockTemplates.filter(template =>
            template.name.toLowerCase().includes(searchLower) ||
            template.description.toLowerCase().includes(searchLower)
          );
        }

        if (options.categoryFilter !== 'all') {
          mockTemplates = mockTemplates.filter(template => template.category === options.categoryFilter);
        }

        setTemplates(mockTemplates);
      } catch (error) {
        console.error('Failed to fetch permission templates:', error);
        setTemplates([]);
      } finally {
        setLoading(false);
      }
    };

    const debounceTimer = setTimeout(fetchTemplates, 300);
    return () => clearTimeout(debounceTimer);
  }, [options.searchTerm, options.categoryFilter]);

  return { templates, loading };
};
