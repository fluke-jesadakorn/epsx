'use client';

import {
  Eye,
  HeadphonesIcon,
  Plus,
  Search,
  Settings,
  Shield,
  Users,
} from 'lucide-react';
import React, { useState, useEffect } from 'react';
import {
  PermissionTemplate,
  usePermissionTemplates,
} from '../../../hooks/iam/usePermissionTemplates';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Badge, Button, Input } from '../../ui/form-components';
import { CreateTemplateModal } from './CreateTemplateModal';
import { TemplatePreviewModal } from './TemplatePreviewModal';
import { DynamicTemplateManagement } from './DynamicTemplateManagement';
import { dynamicTemplateService } from '../../../services/dynamicTemplateService';
import { DynamicTemplate, TemplatePermission, PermissionCategory, PermissionScope } from '@epsx/types';

const CategoryIcon = ({ category }: { category: string }) => {
  const icons = {
    User: <Users className="h-4 w-4" />,
    Admin: <Shield className="h-4 w-4" />,
    Support: <HeadphonesIcon className="h-4 w-4" />,
    Manager: <Settings className="h-4 w-4" />,
  };
  return (
    icons[category as keyof typeof icons] || <Shield className="h-4 w-4" />
  );
};

interface TemplateCardProps {
  template: PermissionTemplate;
  onPreview: () => void;
}

const TemplateCard: React.FC<TemplateCardProps> = ({ template, onPreview }) => (
  <Card className="hover:shadow-md transition-shadow">
    <CardHeader className="pb-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <CategoryIcon category={template.category} />
          <CardTitle className="text-lg">{template.name}</CardTitle>
        </div>
        <Badge variant={template.isActive ? 'default' : 'secondary'}>
          {template.isActive ? 'Active' : 'Inactive'}
        </Badge>
      </div>
      <p className="text-sm text-gray-500">{template.description}</p>
    </CardHeader>
    <CardContent>
      <div className="space-y-3">
        <div className="flex justify-between text-sm">
          <span className="text-gray-500">Category:</span>
          <Badge variant="outline">{template.category}</Badge>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-gray-500">Permissions:</span>
          <span>{template.permissions.length}</span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-gray-500">Users:</span>
          <span>{template.usageCount}</span>
        </div>
        <div className="flex gap-2 pt-2">
          <Button
            variant="outline"
            size="sm"
            className="flex-1"
            onClick={onPreview}
          >
            <Eye className="h-4 w-4 mr-1" />
            Preview
          </Button>
          <Button size="sm" className="flex-1">
            Edit
          </Button>
        </div>
      </div>
    </CardContent>
  </Card>
);

export const PermissionTemplates: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'traditional' | 'dynamic'>('dynamic');
  const [searchTerm, setSearchTerm] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('all');
  const [selectedTemplate, setSelectedTemplate] =
    useState<PermissionTemplate | null>(null);
  const [showPreviewModal, setShowPreviewModal] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);

  // Dynamic template state
  const [dynamicTemplates, setDynamicTemplates] = useState<DynamicTemplate[]>([]);
  const [dynamicLoading, setDynamicLoading] = useState(false);
  const [availablePermissions, setAvailablePermissions] = useState<TemplatePermission[]>([]);

  const { templates, loading, refetch } = usePermissionTemplates({
    searchTerm,
    categoryFilter,
  });

  // Load dynamic templates
  useEffect(() => {
    loadDynamicTemplates();
    loadAvailablePermissions();
  }, []);

  const loadDynamicTemplates = async () => {
    try {
      setDynamicLoading(true);
      const templates = await dynamicTemplateService.listTemplates();
      setDynamicTemplates(templates);
    } catch (error) {
      console.error('Error loading dynamic templates:', error);
    } finally {
      setDynamicLoading(false);
    }
  };

  const loadAvailablePermissions = async () => {
    // Mock available permissions - in real app, this would come from a service
    const permissions: TemplatePermission[] = [
      {
        id: 'users.view',
        name: 'View Users',
        description: 'View user accounts and profiles',
        category: PermissionCategory.ADMIN,
        scope: PermissionScope.GLOBAL,
      },
      {
        id: 'users.create',
        name: 'Create Users',
        description: 'Create new user accounts',
        category: PermissionCategory.ADMIN,
        scope: PermissionScope.GLOBAL,
      },
      {
        id: 'analytics.view',
        name: 'View Analytics',
        description: 'Access analytics dashboard',
        category: PermissionCategory.ANALYTICS,
        scope: PermissionScope.GLOBAL,
      },
      {
        id: 'billing.manage',
        name: 'Manage Billing',
        description: 'Access billing and payment information',
        category: PermissionCategory.BILLING,
        scope: PermissionScope.GLOBAL,
      },
    ];
    setAvailablePermissions(permissions);
  };

  const handlePreview = (template: PermissionTemplate) => {
    setSelectedTemplate(template);
    setShowPreviewModal(true);
  };

  const handleCreateTemplate = async (templateData: any) => {
    try {
      // TODO: Implement template creation via service
      console.log('Creating template:', templateData);
      refetch();
    } catch (error) {
      console.error('Error creating template:', error);
      throw error;
    }
  };

  // Dynamic template handlers
  const handleCreateDynamicTemplate = async (templateData: Omit<DynamicTemplate, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'>) => {
    try {
      await dynamicTemplateService.createTemplate(templateData);
      await loadDynamicTemplates();
    } catch (error) {
      console.error('Error creating dynamic template:', error);
      throw error;
    }
  };

  const handleUpdateDynamicTemplate = async (id: string, templateData: Partial<DynamicTemplate>) => {
    try {
      await dynamicTemplateService.updateTemplate(id, templateData);
      await loadDynamicTemplates();
    } catch (error) {
      console.error('Error updating dynamic template:', error);
      throw error;
    }
  };

  const handleDeleteDynamicTemplate = async (id: string) => {
    try {
      await dynamicTemplateService.deleteTemplate(id);
      await loadDynamicTemplates();
    } catch (error) {
      console.error('Error deleting dynamic template:', error);
      throw error;
    }
  };

  const handleDuplicateDynamicTemplate = async (id: string) => {
    try {
      const template = await dynamicTemplateService.getTemplate(id);
      if (template) {
        const duplicatedTemplate = {
          ...template,
          name: `${template.name} (Copy)`,
          status: template.status,
        };
        delete (duplicatedTemplate as any).id;
        delete (duplicatedTemplate as any).createdAt;
        delete (duplicatedTemplate as any).updatedAt;
        delete (duplicatedTemplate as any).usageCount;
        delete (duplicatedTemplate as any).assignedUserCount;
        
        await dynamicTemplateService.createTemplate(duplicatedTemplate);
        await loadDynamicTemplates();
      }
    } catch (error) {
      console.error('Error duplicating dynamic template:', error);
      throw error;
    }
  };

  const categories = ['all', 'User', 'Admin', 'Support', 'Manager'];

  return (
    <div className="space-y-6">
      {/* Tab Navigation */}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <div>
              <CardTitle>Permission Templates</CardTitle>
              <p className="text-sm text-gray-600 mt-1">
                Manage traditional and dynamic permission templates
              </p>
            </div>
            <div className="flex items-center gap-2">
              <div className="bg-gray-100 p-1 rounded-lg">
                <button
                  onClick={() => setActiveTab('dynamic')}
                  className={`px-4 py-2 text-sm font-medium rounded-md transition-all ${
                    activeTab === 'dynamic'
                      ? 'bg-white text-blue-600 shadow-sm'
                      : 'text-gray-600 hover:text-gray-900'
                  }`}
                >
                  Dynamic Templates
                </button>
                <button
                  onClick={() => setActiveTab('traditional')}
                  className={`px-4 py-2 text-sm font-medium rounded-md transition-all ${
                    activeTab === 'traditional'
                      ? 'bg-white text-blue-600 shadow-sm'
                      : 'text-gray-600 hover:text-gray-900'
                  }`}
                >
                  Traditional Templates
                </button>
              </div>
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* Dynamic Templates Tab */}
      {activeTab === 'dynamic' && (
        <DynamicTemplateManagement
          templates={dynamicTemplates}
          onCreateTemplate={handleCreateDynamicTemplate}
          onUpdateTemplate={handleUpdateDynamicTemplate}
          onDeleteTemplate={handleDeleteDynamicTemplate}
          onDuplicateTemplate={handleDuplicateDynamicTemplate}
          availablePermissions={availablePermissions}
        />
      )}

      {/* Traditional Templates Tab */}
      {activeTab === 'traditional' && (
        <>
          <Card>
            <CardHeader>
              <div className="flex justify-between items-center">
                <CardTitle>Traditional Permission Templates</CardTitle>
                <Button
                  onClick={() => setShowCreateModal(true)}
                  className="flex items-center gap-2"
                >
                  <Plus className="h-4 w-4" />
                  Create Template
                </Button>
              </div>
              <div className="flex flex-col sm:flex-row gap-4">
                <div className="flex-1 relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
                  <Input
                    placeholder="Search templates..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="pl-10"
                  />
                </div>
                <div className="flex gap-2">
                  <select
                    value={categoryFilter}
                    onChange={(e) => setCategoryFilter(e.target.value)}
                    className="h-10 px-3 py-2 border border-gray-300 rounded-md text-sm"
                  >
                    {categories.map((category) => (
                      <option key={category} value={category}>
                        {category === 'all' ? 'All Categories' : category}
                      </option>
                    ))}
                  </select>
                  <Button>
                    <Shield className="h-4 w-4 mr-2" />
                    New Template
                  </Button>
                </div>
              </div>
            </CardHeader>
          </Card>

          {loading ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {[1, 2, 3, 4, 5, 6].map((i) => (
                <Card key={i} className="animate-pulse">
                  <CardHeader className="space-y-2">
                    <div className="h-4 bg-gray-200 rounded w-3/4"></div>
                    <div className="h-3 bg-gray-200 rounded w-full"></div>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-2">
                      <div className="h-3 bg-gray-200 rounded"></div>
                      <div className="h-3 bg-gray-200 rounded"></div>
                      <div className="h-8 bg-gray-200 rounded mt-4"></div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {templates.map((template: PermissionTemplate) => (
                <TemplateCard
                  key={template.id}
                  template={template}
                  onPreview={() => handlePreview(template)}
                />
              ))}
            </div>
          )}

          {showPreviewModal && selectedTemplate && (
            <TemplatePreviewModal
              template={selectedTemplate}
              open={showPreviewModal}
              onClose={() => setShowPreviewModal(false)}
            />
          )}

          <CreateTemplateModal
            isOpen={showCreateModal}
            onClose={() => setShowCreateModal(false)}
            onSave={handleCreateTemplate}
          />
        </>
      )}
    </div>
  );
};
