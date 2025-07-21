'use client';

import React, { useState, useEffect } from 'react';
import { 
  X, 
  Plus, 
  Shield, 
  Tag,
  AlertCircle,
  CheckCircle,
  Search
} from 'lucide-react';
import { PermissionCategory, PermissionScope } from '../../../types/iam/firestore-iam';

interface Permission {
  id: string;
  name: string;
  description: string;
  category: PermissionCategory;
  action: string;
  resource: string;
  scope: PermissionScope;
}

interface PermissionTemplate {
  id?: string;
  name: string;
  description: string;
  category: string;
  permissions: string[];
  tags: string[];
  isSystem: boolean;
}

interface CreatePermissionTemplateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (template: PermissionTemplate | CreatePermissionTemplateData) => Promise<void>;
  existingTemplate?: PermissionTemplate | null;
  isEditing?: boolean;
}

interface CreatePermissionTemplateData {
  name: string;
  description: string;
  category: string;
  permissions: string[];
  tags: string[];
  isSystem: boolean;
}

// Mock available permissions - in real app, this would come from API
const AVAILABLE_PERMISSIONS: Permission[] = [
  {
    id: 'view_dashboard_basic',
    name: 'View Basic Dashboard',
    description: 'Access to basic dashboard features',
    category: PermissionCategory.DASHBOARD,
    action: 'view',
    resource: 'dashboard:basic',
    scope: PermissionScope.OWN
  },
  {
    id: 'view_dashboard_advanced',
    name: 'View Advanced Dashboard',
    description: 'Access to advanced dashboard analytics',
    category: PermissionCategory.DASHBOARD,
    action: 'view',
    resource: 'dashboard:advanced',
    scope: PermissionScope.OWN
  },
  {
    id: 'execute_api_personal',
    name: 'Personal API Access',
    description: 'Execute personal API endpoints',
    category: PermissionCategory.API,
    action: 'execute',
    resource: 'api:personal',
    scope: PermissionScope.OWN
  },
  {
    id: 'execute_api_company',
    name: 'Company API Access',
    description: 'Execute company-wide API endpoints',
    category: PermissionCategory.API,
    action: 'execute',
    resource: 'api:company',
    scope: PermissionScope.COMPANY
  },
  {
    id: 'view_analytics_basic',
    name: 'Basic Analytics',
    description: 'View basic analytics reports',
    category: PermissionCategory.ANALYTICS,
    action: 'view',
    resource: 'analytics:basic',
    scope: PermissionScope.OWN
  },
  {
    id: 'view_analytics_advanced',
    name: 'Advanced Analytics',
    description: 'View advanced analytics and insights',
    category: PermissionCategory.ANALYTICS,
    action: 'view',
    resource: 'analytics:advanced',
    scope: PermissionScope.COMPANY
  },
  {
    id: 'manage_users',
    name: 'Manage Users',
    description: 'Create, edit, and delete users',
    category: PermissionCategory.ADMIN,
    action: 'manage',
    resource: 'users',
    scope: PermissionScope.COMPANY
  },
  {
    id: 'view_billing',
    name: 'View Billing',
    description: 'Access billing information',
    category: PermissionCategory.BILLING,
    action: 'view',
    resource: 'billing',
    scope: PermissionScope.COMPANY
  },
  {
    id: 'export_data',
    name: 'Export Data',
    description: 'Export data to various formats',
    category: PermissionCategory.DATA,
    action: 'export',
    resource: 'data',
    scope: PermissionScope.OWN
  },
  {
    id: 'create_support_tickets',
    name: 'Create Support Tickets',
    description: 'Create support tickets',
    category: PermissionCategory.SUPPORT,
    action: 'create',
    resource: 'support:ticket',
    scope: PermissionScope.OWN
  }
];

const TEMPLATE_CATEGORIES = [
  'User', 'Admin', 'Support', 'API', 'Analytics', 'Billing', 'Custom'
];

export const CreatePermissionTemplateModal: React.FC<CreatePermissionTemplateModalProps> = ({
  isOpen,
  onClose,
  onSave,
  existingTemplate,
  isEditing = false
}) => {
  const [formData, setFormData] = useState<PermissionTemplate>({
    name: '',
    description: '',
    category: 'Custom',
    permissions: [],
    tags: [],
    isSystem: false
  });
  
  const [newTag, setNewTag] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<PermissionCategory | ''>('');
  const [saving, setSaving] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (existingTemplate) {
      setFormData(existingTemplate);
    } else {
      setFormData({
        name: '',
        description: '',
        category: 'Custom',
        permissions: [],
        tags: [],
        isSystem: false
      });
    }
  }, [existingTemplate, isOpen]);

  const filteredPermissions = AVAILABLE_PERMISSIONS.filter(permission => {
    const matchesSearch = !searchQuery || 
      permission.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      permission.description.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesCategory = !selectedCategory || permission.category === selectedCategory;
    
    return matchesSearch && matchesCategory;
  });

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};
    
    if (!formData.name.trim()) {
      newErrors.name = 'Template name is required';
    }
    
    if (!formData.description.trim()) {
      newErrors.description = 'Description is required';
    }
    
    if (formData.permissions.length === 0) {
      newErrors.permissions = 'At least one permission must be selected';
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSave = async () => {
    if (!validateForm()) return;
    
    setSaving(true);
    try {
      await onSave(formData);
      onClose();
    } catch (error) {
      console.error('Failed to save template:', error);
    } finally {
      setSaving(false);
    }
  };

  const togglePermission = (permissionId: string) => {
    setFormData(prev => ({
      ...prev,
      permissions: prev.permissions.includes(permissionId)
        ? prev.permissions.filter(id => id !== permissionId)
        : [...prev.permissions, permissionId]
    }));
  };

  const addTag = () => {
    if (newTag.trim() && !formData.tags.includes(newTag.trim())) {
      setFormData(prev => ({
        ...prev,
        tags: [...prev.tags, newTag.trim()]
      }));
      setNewTag('');
    }
  };

  const removeTag = (tagToRemove: string) => {
    setFormData(prev => ({
      ...prev,
      tags: prev.tags.filter(tag => tag !== tagToRemove)
    }));
  };

  const getCategoryColor = (category: PermissionCategory) => {
    switch (category) {
      case PermissionCategory.DASHBOARD:
        return 'bg-blue-100 text-blue-800';
      case PermissionCategory.API:
        return 'bg-green-100 text-green-800';
      case PermissionCategory.ANALYTICS:
        return 'bg-purple-100 text-purple-800';
      case PermissionCategory.ADMIN:
        return 'bg-red-100 text-red-800';
      case PermissionCategory.BILLING:
        return 'bg-yellow-100 text-yellow-800';
      case PermissionCategory.DATA:
        return 'bg-indigo-100 text-indigo-800';
      case PermissionCategory.SUPPORT:
        return 'bg-pink-100 text-pink-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getScopeIcon = (scope: PermissionScope) => {
    switch (scope) {
      case PermissionScope.OWN:
        return '👤';
      case PermissionScope.COMPANY:
        return '🏢';
      case PermissionScope.PARTNER:
        return '🤝';
      case PermissionScope.GLOBAL:
        return '🌍';
      default:
        return '🔒';
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {isEditing ? 'Edit Permission Template' : 'Create Permission Template'}
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
              Define a reusable set of permissions for user roles
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <X className="h-6 w-6" />
          </button>
        </div>

        <div className="overflow-y-auto max-h-[calc(90vh-120px)]">
          <div className="p-6 space-y-6">
            {/* Basic Information */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Template Name *
                </label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  className={`w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white ${
                    errors.name ? 'border-red-300' : 'border-gray-300'
                  }`}
                  placeholder="e.g., Premium User Access"
                />
                {errors.name && (
                  <p className="text-sm text-red-600 mt-1 flex items-center">
                    <AlertCircle className="h-4 w-4 mr-1" />
                    {errors.name}
                  </p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Category *
                </label>
                <select
                  value={formData.category}
                  onChange={(e) => setFormData(prev => ({ ...prev, category: e.target.value }))}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                >
                  {TEMPLATE_CATEGORIES.map(category => (
                    <option key={category} value={category}>{category}</option>
                  ))}
                </select>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Description *
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                rows={3}
                className={`w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white ${
                  errors.description ? 'border-red-300' : 'border-gray-300'
                }`}
                placeholder="Describe what this template provides..."
              />
              {errors.description && (
                <p className="text-sm text-red-600 mt-1 flex items-center">
                  <AlertCircle className="h-4 w-4 mr-1" />
                  {errors.description}
                </p>
              )}
            </div>

            {/* Tags */}
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Tags
              </label>
              <div className="flex flex-wrap gap-2 mb-2">
                {formData.tags.map((tag, index) => (
                  <span
                    key={index}
                    className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
                  >
                    <Tag className="h-3 w-3 mr-1" />
                    {tag}
                    <button
                      onClick={() => removeTag(tag)}
                      className="ml-1 text-blue-600 hover:text-blue-800 dark:text-blue-300 dark:hover:text-blue-100"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  </span>
                ))}
              </div>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newTag}
                  onChange={(e) => setNewTag(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && addTag()}
                  className="flex-1 px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                  placeholder="Add a tag..."
                />
                <button
                  onClick={addTag}
                  className="px-3 py-1 bg-blue-600 text-white rounded-md hover:bg-blue-700 text-sm"
                >
                  <Plus className="h-4 w-4" />
                </button>
              </div>
            </div>

            {/* Permissions Selection */}
            <div>
              <div className="flex items-center justify-between mb-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Permissions * ({formData.permissions.length} selected)
                  </label>
                  {errors.permissions && (
                    <p className="text-sm text-red-600 mt-1 flex items-center">
                      <AlertCircle className="h-4 w-4 mr-1" />
                      {errors.permissions}
                    </p>
                  )}
                </div>
              </div>

              {/* Permission Filters */}
              <div className="flex flex-col sm:flex-row gap-4 mb-4">
                <div className="flex-1 relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
                  <input
                    type="text"
                    placeholder="Search permissions..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                  />
                </div>
                
                <select
                  value={selectedCategory}
                  onChange={(e) => setSelectedCategory(e.target.value as PermissionCategory | '')}
                  className="border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                >
                  <option value="">All Categories</option>
                  {Object.values(PermissionCategory).map(category => (
                    <option key={category} value={category}>
                      {category.charAt(0).toUpperCase() + category.slice(1)}
                    </option>
                  ))}
                </select>
              </div>

              {/* Permissions List */}
              <div className="border border-gray-200 dark:border-gray-600 rounded-lg max-h-64 overflow-y-auto">
                {filteredPermissions.map((permission) => (
                  <div
                    key={permission.id}
                    className={`p-3 border-b border-gray-100 dark:border-gray-700 last:border-b-0 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 ${
                      formData.permissions.includes(permission.id) ? 'bg-blue-50 dark:bg-blue-900/20' : ''
                    }`}
                    onClick={() => togglePermission(permission.id)}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <input
                            type="checkbox"
                            checked={formData.permissions.includes(permission.id)}
                            onChange={() => togglePermission(permission.id)}
                            className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                          />
                          <h4 className="font-medium text-gray-900 dark:text-white text-sm">
                            {permission.name}
                          </h4>
                          <span className="text-lg" title={`Scope: ${permission.scope}`}>
                            {getScopeIcon(permission.scope)}
                          </span>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 ml-6">
                          {permission.description}
                        </p>
                        <div className="flex items-center gap-2 mt-2 ml-6">
                          <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${getCategoryColor(permission.category)}`}>
                            {permission.category}
                          </span>
                          <span className="text-xs text-gray-400 font-mono">
                            {permission.action}:{permission.resource}
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
                
                {filteredPermissions.length === 0 && (
                  <div className="p-8 text-center text-gray-500 dark:text-gray-400">
                    <Shield className="h-8 w-8 mx-auto mb-2 opacity-50" />
                    <p>No permissions found matching your criteria</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t border-gray-200 dark:border-gray-700">
          <div className="text-sm text-gray-500 dark:text-gray-400">
            {formData.permissions.length > 0 && (
              <span className="flex items-center">
                <CheckCircle className="h-4 w-4 text-green-500 mr-1" />
                {formData.permissions.length} permission{formData.permissions.length !== 1 ? 's' : ''} selected
              </span>
            )}
          </div>
          
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={saving || formData.permissions.length === 0}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
            >
              {saving ? (
                <span className="flex items-center">
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                  Saving...
                </span>
              ) : (
                <>
                  <Shield className="h-4 w-4 mr-2" />
                  {isEditing ? 'Update Template' : 'Create Template'}
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
