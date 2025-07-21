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
import React, { useState } from 'react';
import {
  PermissionTemplate,
  usePermissionTemplates,
} from '../../../hooks/iam/usePermissionTemplates';
import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
import { Badge, Button, Input } from '../../ui/form-components';
import { CreateTemplateModal } from './CreateTemplateModal';
import { TemplatePreviewModal } from './TemplatePreviewModal';

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
  const [searchTerm, setSearchTerm] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('all');
  const [selectedTemplate, setSelectedTemplate] =
    useState<PermissionTemplate | null>(null);
  const [showPreviewModal, setShowPreviewModal] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);

  const { templates, loading, refetch } = usePermissionTemplates({
    searchTerm,
    categoryFilter,
  });

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

  const categories = ['all', 'User', 'Admin', 'Support', 'Manager'];

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle>Permission Templates</CardTitle>
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
    </div>
  );
};
