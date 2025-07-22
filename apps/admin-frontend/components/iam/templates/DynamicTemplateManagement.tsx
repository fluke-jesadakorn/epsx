'use client';

import React, { useState, useEffect } from 'react';
import {
  Shield,
  Plus,
  Search,
  Filter,
  MoreVertical,
  Edit,
  Trash2,
  Copy,
  Users,
  Eye,
  Archive,
  CheckCircle,
  AlertTriangle,
  Clock,
  Globe,
  Lock,
  Building
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '../../ui/form-components';
import {
  DynamicTemplate,
  TemplateScope,
  TemplateStatus,
  PackageTier,
  PermissionCategory
} from '@epsx/types';
import { DynamicTemplateBuilder } from './DynamicTemplateBuilder';

interface DynamicTemplateManagementProps {
  templates: DynamicTemplate[];
  onCreateTemplate: (template: Omit<DynamicTemplate, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'>) => Promise<void>;
  onUpdateTemplate: (id: string, template: Partial<DynamicTemplate>) => Promise<void>;
  onDeleteTemplate: (id: string) => Promise<void>;
  onDuplicateTemplate: (id: string) => Promise<void>;
  availablePermissions: any[];
}

export const DynamicTemplateManagement: React.FC<DynamicTemplateManagementProps> = ({
  templates,
  onCreateTemplate,
  onUpdateTemplate,
  onDeleteTemplate,
  onDuplicateTemplate,
  availablePermissions,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [filterScope, setFilterScope] = useState<TemplateScope | 'all'>('all');
  const [filterStatus, setFilterStatus] = useState<TemplateStatus | 'all'>('all');
  const [showBuilder, setShowBuilder] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<DynamicTemplate | undefined>();
  const [selectedTemplates, setSelectedTemplates] = useState<string[]>([]);

  // Filter templates based on search and filters
  const filteredTemplates = templates.filter(template => {
    const matchesSearch = template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         template.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         template.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
    
    const matchesScope = filterScope === 'all' || template.scope === filterScope;
    const matchesStatus = filterStatus === 'all' || template.status === filterStatus;
    
    return matchesSearch && matchesScope && matchesStatus;
  });

  // Handle template creation
  const handleCreateTemplate = async (templateData: Omit<DynamicTemplate, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'>) => {
    await onCreateTemplate(templateData);
    setShowBuilder(false);
  };

  // Handle template editing
  const handleEditTemplate = (template: DynamicTemplate) => {
    setEditingTemplate(template);
    setShowBuilder(true);
  };

  // Handle template update
  const handleUpdateTemplate = async (templateData: Omit<DynamicTemplate, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'>) => {
    if (editingTemplate) {
      await onUpdateTemplate(editingTemplate.id, templateData);
      setEditingTemplate(undefined);
      setShowBuilder(false);
    }
  };

  // Handle bulk actions
  const handleBulkAction = async (action: 'delete' | 'archive' | 'activate') => {
    for (const templateId of selectedTemplates) {
      switch (action) {
        case 'delete':
          await onDeleteTemplate(templateId);
          break;
        case 'archive':
          await onUpdateTemplate(templateId, { status: TemplateStatus.ARCHIVED });
          break;
        case 'activate':
          await onUpdateTemplate(templateId, { status: TemplateStatus.ACTIVE });
          break;
      }
    }
    setSelectedTemplates([]);
  };

  // Get status icon and color
  const getStatusDisplay = (status: TemplateStatus) => {
    switch (status) {
      case TemplateStatus.ACTIVE:
        return { icon: <CheckCircle className="h-4 w-4" />, color: 'text-green-600', bg: 'bg-green-100' };
      case TemplateStatus.DRAFT:
        return { icon: <Clock className="h-4 w-4" />, color: 'text-yellow-600', bg: 'bg-yellow-100' };
      case TemplateStatus.ARCHIVED:
        return { icon: <Archive className="h-4 w-4" />, color: 'text-gray-600', bg: 'bg-gray-100' };
      case TemplateStatus.DEPRECATED:
        return { icon: <AlertTriangle className="h-4 w-4" />, color: 'text-red-600', bg: 'bg-red-100' };
      default:
        return { icon: <Clock className="h-4 w-4" />, color: 'text-gray-600', bg: 'bg-gray-100' };
    }
  };

  // Get scope icon
  const getScopeIcon = (scope: TemplateScope) => {
    switch (scope) {
      case TemplateScope.SYSTEM:
        return <Shield className="h-4 w-4" />;
      case TemplateScope.ORGANIZATION:
        return <Building className="h-4 w-4" />;
      case TemplateScope.PERSONAL:
        return <Lock className="h-4 w-4" />;
      default:
        return <Lock className="h-4 w-4" />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold flex items-center gap-2">
            <Shield className="h-6 w-6" />
            Dynamic Templates
          </h2>
          <p className="text-gray-600 mt-1">
            Create and manage custom permission templates
          </p>
        </div>
        <Button
          onClick={() => {
            setEditingTemplate(undefined);
            setShowBuilder(true);
          }}
          className="flex items-center gap-2"
        >
          <Plus className="h-4 w-4" />
          Create Template
        </Button>
      </div>

      {/* Filters and Search */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
              <input
                type="text"
                placeholder="Search templates by name, description, or tags..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
              />
            </div>
            
            <select
              className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              value={filterScope}
              onChange={(e) => setFilterScope(e.target.value as TemplateScope | 'all')}
            >
              <option value="all">All Scopes</option>
              <option value={TemplateScope.SYSTEM}>System</option>
              <option value={TemplateScope.ORGANIZATION}>Organization</option>
              <option value={TemplateScope.PERSONAL}>Personal</option>
            </select>

            <select
              className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value as TemplateStatus | 'all')}
            >
              <option value="all">All Status</option>
              <option value={TemplateStatus.ACTIVE}>Active</option>
              <option value={TemplateStatus.DRAFT}>Draft</option>
              <option value={TemplateStatus.ARCHIVED}>Archived</option>
              <option value={TemplateStatus.DEPRECATED}>Deprecated</option>
            </select>
          </div>

          {selectedTemplates.length > 0 && (
            <div className="mt-4 p-3 bg-blue-50 rounded-md flex items-center justify-between">
              <span className="text-sm text-blue-800">
                {selectedTemplates.length} template(s) selected
              </span>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => handleBulkAction('activate')}
                >
                  Activate
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => handleBulkAction('archive')}
                >
                  Archive
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => handleBulkAction('delete')}
                  className="text-red-600 border-red-300 hover:bg-red-50"
                >
                  Delete
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Templates Grid */}
      <div className="grid gap-4">
        {filteredTemplates.length === 0 ? (
          <Card>
            <CardContent className="p-8 text-center">
              <Shield className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No templates found</h3>
              <p className="text-gray-600 mb-4">
                {searchTerm || filterScope !== 'all' || filterStatus !== 'all'
                  ? 'Try adjusting your search or filters'
                  : 'Create your first dynamic template to get started'}
              </p>
              {!searchTerm && filterScope === 'all' && filterStatus === 'all' && (
                <Button
                  onClick={() => {
                    setEditingTemplate(undefined);
                    setShowBuilder(true);
                  }}
                  className="flex items-center gap-2"
                >
                  <Plus className="h-4 w-4" />
                  Create Template
                </Button>
              )}
            </CardContent>
          </Card>
        ) : (
          filteredTemplates.map((template) => {
            const statusDisplay = getStatusDisplay(template.status);
            const isSelected = selectedTemplates.includes(template.id);

            return (
              <Card
                key={template.id}
                className={`transition-all hover:shadow-md ${
                  isSelected ? 'ring-2 ring-blue-500 bg-blue-50' : ''
                }`}
              >
                <CardContent className="p-6">
                  <div className="flex items-start gap-4">
                    {/* Selection Checkbox */}
                    <input
                      type="checkbox"
                      className="mt-1"
                      checked={isSelected}
                      onChange={(e) => {
                        if (e.target.checked) {
                          setSelectedTemplates([...selectedTemplates, template.id]);
                        } else {
                          setSelectedTemplates(selectedTemplates.filter(id => id !== template.id));
                        }
                      }}
                    />

                    {/* Template Info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between mb-2">
                        <div className="flex items-center gap-2">
                          <h3 className="text-lg font-medium truncate">{template.name}</h3>
                          <div className="flex items-center gap-1">
                            {getScopeIcon(template.scope)}
                            {template.isPublic && <Globe className="h-4 w-4 text-blue-500" />}
                          </div>
                        </div>
                        
                        <div className="flex items-center gap-2">
                          <div className={`flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium ${statusDisplay.bg} ${statusDisplay.color}`}>
                            {statusDisplay.icon}
                            {template.status}
                          </div>
                          
                          <div className="relative group">
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-8 w-8"
                            >
                              <MoreVertical className="h-4 w-4" />
                            </Button>
                            
                            {/* Dropdown Menu */}
                            <div className="absolute right-0 top-8 w-48 bg-white rounded-md shadow-lg border border-gray-200 z-10 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all">
                              <div className="py-1">
                                <button
                                  className="w-full px-4 py-2 text-sm text-left hover:bg-gray-50 flex items-center gap-2"
                                  onClick={() => handleEditTemplate(template)}
                                >
                                  <Edit className="h-4 w-4" />
                                  Edit
                                </button>
                                <button
                                  className="w-full px-4 py-2 text-sm text-left hover:bg-gray-50 flex items-center gap-2"
                                  onClick={() => onDuplicateTemplate(template.id)}
                                >
                                  <Copy className="h-4 w-4" />
                                  Duplicate
                                </button>
                                <button
                                  className="w-full px-4 py-2 text-sm text-left hover:bg-gray-50 flex items-center gap-2"
                                >
                                  <Eye className="h-4 w-4" />
                                  Preview
                                </button>
                                <hr className="my-1" />
                                <button
                                  className="w-full px-4 py-2 text-sm text-left hover:bg-gray-50 flex items-center gap-2 text-red-600"
                                  onClick={() => onDeleteTemplate(template.id)}
                                >
                                  <Trash2 className="h-4 w-4" />
                                  Delete
                                </button>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>

                      <p className="text-gray-600 text-sm mb-3 line-clamp-2">
                        {template.description}
                      </p>

                      {/* Template Stats */}
                      <div className="flex items-center gap-4 text-sm text-gray-500 mb-3">
                        <div className="flex items-center gap-1">
                          <Shield className="h-4 w-4" />
                          {template.permissions.length} permissions
                        </div>
                        <div className="flex items-center gap-1">
                          <Users className="h-4 w-4" />
                          {template.assignedUserCount} users
                        </div>
                        <div className="flex items-center gap-1">
                          <Eye className="h-4 w-4" />
                          {template.usageCount} uses
                        </div>
                        <div>
                          v{template.version}
                        </div>
                      </div>

                      {/* Tags and Categories */}
                      <div className="flex flex-wrap gap-2">
                        {template.categories.map(category => (
                          <span
                            key={category}
                            className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full"
                          >
                            {category}
                          </span>
                        ))}
                        {template.tags.map(tag => (
                          <span
                            key={tag}
                            className="px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded-full"
                          >
                            #{tag}
                          </span>
                        ))}
                      </div>

                      {/* Package Compatibility */}
                      {template.packageTierCompatibility.length > 0 && (
                        <div className="mt-3 pt-3 border-t border-gray-100">
                          <div className="text-xs text-gray-500 mb-1">Compatible with:</div>
                          <div className="flex flex-wrap gap-1">
                            {template.packageTierCompatibility.map(tier => (
                              <span
                                key={tier}
                                className="px-2 py-0.5 bg-green-100 text-green-800 text-xs rounded"
                              >
                                {tier}
                              </span>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })
        )}
      </div>

      {/* Template Builder Modal */}
      <DynamicTemplateBuilder
        isOpen={showBuilder}
        onClose={() => {
          setShowBuilder(false);
          setEditingTemplate(undefined);
        }}
        onSave={editingTemplate ? handleUpdateTemplate : handleCreateTemplate}
        editingTemplate={editingTemplate}
        availablePermissions={availablePermissions}
      />
    </div>
  );
};