'use client';

import React, { useState } from 'react';
import { 
  Plus, 
  Edit, 
  Trash2, 
  Shield, 
  Users, 
  Search,
  MoreHorizontal,
  Eye,
  Copy,
  CheckCircle,
  SortAsc,
  SortDesc,
  RefreshCw
} from 'lucide-react';
import { CreatePermissionTemplateModal } from './CreatePermissionTemplateModal';
import { ViewPermissionTemplateModal } from './ViewPermissionTemplateModal';
import { DeleteConfirmationModal } from './DeleteConfirmationModal';

interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  permissions: string[];
  tags: string[];
  usageCount: number;
  isSystem: boolean;
  createdAt: Date;
  updatedAt: Date;
}

interface CreatePermissionTemplateData {
  name: string;
  description: string;
  category: string;
  permissions: string[];
  tags: string[];
  isSystem: boolean;
}

type SortField = 'name' | 'usageCount' | 'updatedAt' | 'category';
type SortDirection = 'asc' | 'desc';

export const PermissionTemplates: React.FC = () => {
  const [templates, setTemplates] = useState<PermissionTemplate[]>([
    {
      id: '1',
      name: 'Basic User Access',
      description: 'Standard permissions for regular users including dashboard access and profile management',
      category: 'User',
      permissions: ['view_dashboard_basic', 'view_analytics_basic', 'export_data', 'create_support_tickets'],
      tags: ['standard', 'user', 'basic'],
      usageCount: 156,
      isSystem: true,
      createdAt: new Date('2024-01-15'),
      updatedAt: new Date('2024-03-10')
    },
    {
      id: '2',
      name: 'Premium User Access',
      description: 'Enhanced permissions for premium users with advanced analytics and API access',
      category: 'User',
      permissions: ['view_dashboard_advanced', 'view_analytics_advanced', 'execute_api_personal', 'export_data'],
      tags: ['premium', 'api', 'analytics'],
      usageCount: 89,
      isSystem: true,
      createdAt: new Date('2024-01-15'),
      updatedAt: new Date('2024-03-10')
    },
    {
      id: '3',
      name: 'Support Agent',
      description: 'Permissions for customer support team members to assist users',
      category: 'Support',
      permissions: ['view_dashboard_basic', 'manage_users', 'view_billing'],
      tags: ['support', 'customer-service'],
      usageCount: 12,
      isSystem: false,
      createdAt: new Date('2024-02-01'),
      updatedAt: new Date('2024-03-15')
    },
    {
      id: '4',
      name: 'API Developer',
      description: 'Full API access for developers and integrations',
      category: 'API',
      permissions: ['execute_api_personal', 'execute_api_company', 'view_analytics_advanced'],
      tags: ['api', 'developer', 'integration'],
      usageCount: 23,
      isSystem: false,
      createdAt: new Date('2024-02-20'),
      updatedAt: new Date('2024-03-12')
    },
    {
      id: '5',
      name: 'Analytics Specialist',
      description: 'Access to all analytics and reporting features',
      category: 'Analytics',
      permissions: ['view_analytics_basic', 'view_analytics_advanced', 'export_data', 'view_dashboard_advanced'],
      tags: ['analytics', 'reporting', 'data'],
      usageCount: 8,
      isSystem: false,
      createdAt: new Date('2024-03-01'),
      updatedAt: new Date('2024-03-20')
    }
  ]);
  
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('');
  const [sortField, setSortField] = useState<SortField>('updatedAt');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [loading, setLoading] = useState(false);
  
  // Modal states
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showViewModal, setShowViewModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<PermissionTemplate | null>(null);
  const [editingTemplate, setEditingTemplate] = useState<PermissionTemplate | null>(null);
  const [deletingTemplate, setDeletingTemplate] = useState<PermissionTemplate | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const categories = ['All Categories', 'User', 'Admin', 'Support', 'API', 'Analytics', 'Billing', 'Custom'];
  
  const filteredAndSortedTemplates = templates
    .filter(template => {
      const matchesSearch = !searchQuery || 
        template.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        template.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        template.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()));
      
      const matchesCategory = !selectedCategory || 
        selectedCategory === 'All Categories' || 
        template.category === selectedCategory;
      
      return matchesSearch && matchesCategory;
    })
    .sort((a, b) => {
      let comparison = 0;
      
      switch (sortField) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'usageCount':
          comparison = a.usageCount - b.usageCount;
          break;
        case 'updatedAt':
          comparison = a.updatedAt.getTime() - b.updatedAt.getTime();
          break;
        case 'category':
          comparison = a.category.localeCompare(b.category);
          break;
        default:
          comparison = 0;
      }
      
      return sortDirection === 'asc' ? comparison : -comparison;
    });

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('asc');
    }
  };

  const handleCreateTemplate = async (templateData: CreatePermissionTemplateData) => {
    setLoading(true);
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const newTemplate: PermissionTemplate = {
        ...templateData,
        id: Date.now().toString(),
        usageCount: 0,
        createdAt: new Date(),
        updatedAt: new Date()
      };
      
      setTemplates(prev => [...prev, newTemplate]);
    } catch (error) {
      console.error('Failed to create template:', error);
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const handleEditTemplate = async (template: PermissionTemplate) => {
    setLoading(true);
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const updatedTemplate = {
        ...template,
        updatedAt: new Date()
      };
      
      setTemplates(prev => 
        prev.map(t => t.id === template.id ? updatedTemplate : t)
      );
      setEditingTemplate(null);
    } catch (error) {
      console.error('Failed to update template:', error);
      throw error;
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteTemplate = async () => {
    if (!deletingTemplate) return;
    
    setIsDeleting(true);
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      setTemplates(prev => prev.filter(t => t.id !== deletingTemplate.id));
      setShowDeleteModal(false);
      setDeletingTemplate(null);
    } catch (error) {
      console.error('Failed to delete template:', error);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleDuplicateTemplate = async (template: PermissionTemplate) => {
    const duplicatedTemplate: PermissionTemplate = {
      ...template,
      id: `${Date.now()}`,
      name: `${template.name} (Copy)`,
      isSystem: false,
      usageCount: 0,
      createdAt: new Date(),
      updatedAt: new Date()
    };
    
    setEditingTemplate(duplicatedTemplate);
    setShowCreateModal(true);
  };

  const handleViewTemplate = (template: PermissionTemplate) => {
    setSelectedTemplate(template);
    setShowViewModal(true);
  };

  const handleEditFromView = (template: PermissionTemplate) => {
    setShowViewModal(false);
    setEditingTemplate(template);
    setShowCreateModal(true);
  };

  const handleDeleteFromView = (template: PermissionTemplate) => {
    setShowViewModal(false);
    setDeletingTemplate(template);
    setShowDeleteModal(true);
  };

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'User':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
      case 'Admin':
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
      case 'Support':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
      case 'API':
        return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300';
      case 'Analytics':
        return 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900/20 dark:text-indigo-300';
      case 'Billing':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
    }
  };

  const getSortIcon = (field: SortField) => {
    if (sortField !== field) return null;
    return sortDirection === 'asc' ? <SortAsc className="h-4 w-4" /> : <SortDesc className="h-4 w-4" />;
  };

  const handleSaveTemplate = async (templateData: CreatePermissionTemplateData | PermissionTemplate) => {
    if ('id' in templateData) {
      // Editing existing template
      await handleEditTemplate(templateData as PermissionTemplate);
    } else {
      // Creating new template
      await handleCreateTemplate(templateData as CreatePermissionTemplateData);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row gap-4 justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-600" />
            Permission Templates
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Create and manage reusable permission templates for quick user setup
          </p>
        </div>
        
        <div className="flex gap-2">
          <button
            onClick={() => setLoading(!loading)}
            className="inline-flex items-center px-3 py-2 border border-gray-300 dark:border-gray-600 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </button>
          <button
            onClick={() => {
              setEditingTemplate(null);
              setShowCreateModal(true);
            }}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-colors"
          >
            <Plus className="h-4 w-4 mr-2" />
            Create Template
          </button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <Shield className="h-8 w-8 text-blue-600" />
            </div>
            <div className="ml-4">
              <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Total Templates</div>
              <div className="text-2xl font-semibold text-gray-900 dark:text-white">{templates.length}</div>
            </div>
          </div>
        </div>
        
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <Users className="h-8 w-8 text-green-600" />
            </div>
            <div className="ml-4">
              <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Total Usage</div>
              <div className="text-2xl font-semibold text-gray-900 dark:text-white">
                {templates.reduce((sum, t) => sum + t.usageCount, 0)}
              </div>
            </div>
          </div>
        </div>
        
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <CheckCircle className="h-8 w-8 text-purple-600" />
            </div>
            <div className="ml-4">
              <div className="text-sm font-medium text-gray-500 dark:text-gray-400">System Templates</div>
              <div className="text-2xl font-semibold text-gray-900 dark:text-white">
                {templates.filter(t => t.isSystem).length}
              </div>
            </div>
          </div>
        </div>
        
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <Edit className="h-8 w-8 text-orange-600" />
            </div>
            <div className="ml-4">
              <div className="text-sm font-medium text-gray-500 dark:text-gray-400">Custom Templates</div>
              <div className="text-2xl font-semibold text-gray-900 dark:text-white">
                {templates.filter(t => !t.isSystem).length}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Search and Filter */}
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
        <div className="flex flex-col sm:flex-row gap-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
            <input
              type="text"
              placeholder="Search templates, descriptions, or tags..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
          
          <div className="flex gap-2">
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              className="border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            >
              {categories.map((category) => (
                <option key={category} value={category === 'All Categories' ? '' : category}>
                  {category}
                </option>
              ))}
            </select>
            
            <div className="flex border border-gray-300 dark:border-gray-600 rounded-lg overflow-hidden">
              <button
                onClick={() => handleSort('name')}
                className={`px-3 py-2 text-sm flex items-center gap-1 hover:bg-gray-50 dark:hover:bg-gray-600 ${
                  sortField === 'name' ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400' : 'text-gray-700 dark:text-gray-300'
                }`}
              >
                Name {getSortIcon('name')}
              </button>
              <button
                onClick={() => handleSort('usageCount')}
                className={`px-3 py-2 text-sm flex items-center gap-1 hover:bg-gray-50 dark:hover:bg-gray-600 border-l border-gray-300 dark:border-gray-600 ${
                  sortField === 'usageCount' ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400' : 'text-gray-700 dark:text-gray-300'
                }`}
              >
                Usage {getSortIcon('usageCount')}
              </button>
              <button
                onClick={() => handleSort('updatedAt')}
                className={`px-3 py-2 text-sm flex items-center gap-1 hover:bg-gray-50 dark:hover:bg-gray-600 border-l border-gray-300 dark:border-gray-600 ${
                  sortField === 'updatedAt' ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400' : 'text-gray-700 dark:text-gray-300'
                }`}
              >
                Updated {getSortIcon('updatedAt')}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Templates Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredAndSortedTemplates.map((template) => (
          <div key={template.id} className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm hover:shadow-md transition-shadow">
            <div className="p-6">
              <div className="flex items-start justify-between mb-4">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <h4 className="font-medium text-gray-900 dark:text-white">{template.name}</h4>
                    {template.isSystem && (
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300">
                        System
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mb-3">{template.description}</p>
                  
                  <div className="flex items-center gap-2 mb-3">
                    <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${getCategoryColor(template.category)}`}>
                      {template.category}
                    </span>
                    <span className="inline-flex items-center text-xs text-gray-500 dark:text-gray-400">
                      <Users className="h-3 w-3 mr-1" />
                      {template.usageCount} users
                    </span>
                  </div>

                  {/* Tags */}
                  {template.tags.length > 0 && (
                    <div className="flex flex-wrap gap-1 mb-3">
                      {template.tags.slice(0, 3).map((tag, index) => (
                        <span
                          key={index}
                          className="inline-flex items-center px-1.5 py-0.5 rounded text-xs bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300"
                        >
                          {tag}
                        </span>
                      ))}
                      {template.tags.length > 3 && (
                        <span className="text-xs text-gray-400 dark:text-gray-500">
                          +{template.tags.length - 3} more
                        </span>
                      )}
                    </div>
                  )}
                </div>
                
                <div className="relative">
                  <button className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
                    <MoreHorizontal className="h-5 w-5" />
                  </button>
                </div>
              </div>

              {/* Permissions Preview */}
              <div className="mb-4">
                <h5 className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-2 flex items-center">
                  <Shield className="h-3 w-3 mr-1" />
                  Permissions ({template.permissions.length})
                </h5>
                <div className="space-y-1">
                  {template.permissions.slice(0, 3).map((permission, index) => (
                    <div key={index} className="text-xs text-gray-500 dark:text-gray-400 flex items-center">
                      <span className="w-2 h-2 bg-blue-400 rounded-full mr-2"></span>
                      <span className="font-mono">{permission}</span>
                    </div>
                  ))}
                  {template.permissions.length > 3 && (
                    <div className="text-xs text-gray-400 dark:text-gray-500">
                      +{template.permissions.length - 3} more...
                    </div>
                  )}
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center justify-between pt-4 border-t border-gray-100 dark:border-gray-600">
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  Updated {template.updatedAt.toLocaleDateString()}
                </div>
                
                <div className="flex items-center gap-2">
                  <button 
                    onClick={() => handleViewTemplate(template)}
                    className="text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
                    title="View Details"
                  >
                    <Eye className="h-4 w-4" />
                  </button>
                  <button 
                    onClick={() => handleDuplicateTemplate(template)}
                    className="text-gray-400 hover:text-green-600 dark:hover:text-green-400 transition-colors"
                    title="Duplicate Template"
                  >
                    <Copy className="h-4 w-4" />
                  </button>
                  {!template.isSystem && (
                    <>
                      <button 
                        onClick={() => {
                          setEditingTemplate(template);
                          setShowCreateModal(true);
                        }}
                        className="text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
                        title="Edit Template"
                      >
                        <Edit className="h-4 w-4" />
                      </button>
                      <button 
                        onClick={() => {
                          setDeletingTemplate(template);
                          setShowDeleteModal(true);
                        }}
                        className="text-gray-400 hover:text-red-600 dark:hover:text-red-400 transition-colors"
                        title="Delete Template"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </>
                  )}
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {filteredAndSortedTemplates.length === 0 && (
        <div className="text-center py-12">
          <Shield className="h-12 w-12 text-gray-400 dark:text-gray-500 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">No templates found</h3>
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            {searchQuery || selectedCategory 
              ? 'Try adjusting your search or filters' 
              : 'Create your first permission template to get started'
            }
          </p>
          {!searchQuery && !selectedCategory && (
            <button
              onClick={() => {
                setEditingTemplate(null);
                setShowCreateModal(true);
              }}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              <Plus className="h-4 w-4 mr-2" />
              Create Template
            </button>
          )}
        </div>
      )}

      {/* Modals */}
      <CreatePermissionTemplateModal
        isOpen={showCreateModal}
        onClose={() => {
          setShowCreateModal(false);
          setEditingTemplate(null);
        }}
        onSave={handleSaveTemplate}
        existingTemplate={editingTemplate}
        isEditing={!!editingTemplate}
      />

      <ViewPermissionTemplateModal
        isOpen={showViewModal}
        onClose={() => {
          setShowViewModal(false);
          setSelectedTemplate(null);
        }}
        template={selectedTemplate}
        onEdit={handleEditFromView}
        onDuplicate={handleDuplicateTemplate}
        onDelete={handleDeleteFromView}
      />

      <DeleteConfirmationModal
        isOpen={showDeleteModal}
        onClose={() => {
          setShowDeleteModal(false);
          setDeletingTemplate(null);
        }}
        onConfirm={handleDeleteTemplate}
        templateName={deletingTemplate?.name || ''}
        usageCount={deletingTemplate?.usageCount || 0}
        isDeleting={isDeleting}
      />
    </div>
  );
};
