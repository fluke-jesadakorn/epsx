'use client';

import React from 'react';
import { 
  X, 
  Shield, 
  Users, 
  Tag,
  Calendar,
  Eye,
  Copy,
  Edit,
  AlertTriangle
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
  id: string;
  name: string;
  description: string;
  category: string;
  permissions: string[];
  tags: string[];
  isSystem: boolean;
  usageCount: number;
  createdAt: Date;
  updatedAt: Date;
}

interface ViewPermissionTemplateModalProps {
  isOpen: boolean;
  onClose: () => void;
  template: PermissionTemplate | null;
  onEdit?: (template: PermissionTemplate) => void;
  onDuplicate?: (template: PermissionTemplate) => void;
  onDelete?: (template: PermissionTemplate) => void;
}

// Mock permissions data - in real app, this would come from API
const AVAILABLE_PERMISSIONS: Record<string, Permission> = {
  'view_dashboard_basic': {
    id: 'view_dashboard_basic',
    name: 'View Basic Dashboard',
    description: 'Access to basic dashboard features',
    category: PermissionCategory.DASHBOARD,
    action: 'view',
    resource: 'dashboard:basic',
    scope: PermissionScope.OWN
  },
  'view_dashboard_advanced': {
    id: 'view_dashboard_advanced',
    name: 'View Advanced Dashboard',
    description: 'Access to advanced dashboard analytics',
    category: PermissionCategory.DASHBOARD,
    action: 'view',
    resource: 'dashboard:advanced',
    scope: PermissionScope.OWN
  },
  'execute_api_personal': {
    id: 'execute_api_personal',
    name: 'Personal API Access',
    description: 'Execute personal API endpoints',
    category: PermissionCategory.API,
    action: 'execute',
    resource: 'api:personal',
    scope: PermissionScope.OWN
  },
  'execute_api_company': {
    id: 'execute_api_company',
    name: 'Company API Access',
    description: 'Execute company-wide API endpoints',
    category: PermissionCategory.API,
    action: 'execute',
    resource: 'api:company',
    scope: PermissionScope.COMPANY
  },
  'view_analytics_basic': {
    id: 'view_analytics_basic',
    name: 'Basic Analytics',
    description: 'View basic analytics reports',
    category: PermissionCategory.ANALYTICS,
    action: 'view',
    resource: 'analytics:basic',
    scope: PermissionScope.OWN
  },
  'view_analytics_advanced': {
    id: 'view_analytics_advanced',
    name: 'Advanced Analytics',
    description: 'View advanced analytics and insights',
    category: PermissionCategory.ANALYTICS,
    action: 'view',
    resource: 'analytics:advanced',
    scope: PermissionScope.COMPANY
  },
  'manage_users': {
    id: 'manage_users',
    name: 'Manage Users',
    description: 'Create, edit, and delete users',
    category: PermissionCategory.ADMIN,
    action: 'manage',
    resource: 'users',
    scope: PermissionScope.COMPANY
  },
  'view_billing': {
    id: 'view_billing',
    name: 'View Billing',
    description: 'Access billing information',
    category: PermissionCategory.BILLING,
    action: 'view',
    resource: 'billing',
    scope: PermissionScope.COMPANY
  },
  'export_data': {
    id: 'export_data',
    name: 'Export Data',
    description: 'Export data to various formats',
    category: PermissionCategory.DATA,
    action: 'export',
    resource: 'data',
    scope: PermissionScope.OWN
  },
  'create_support_tickets': {
    id: 'create_support_tickets',
    name: 'Create Support Tickets',
    description: 'Create support tickets',
    category: PermissionCategory.SUPPORT,
    action: 'create',
    resource: 'support:ticket',
    scope: PermissionScope.OWN
  }
};

export const ViewPermissionTemplateModal: React.FC<ViewPermissionTemplateModalProps> = ({
  isOpen,
  onClose,
  template,
  onEdit,
  onDuplicate,
  onDelete
}) => {
  if (!isOpen || !template) return null;

  const getCategoryColor = (category: PermissionCategory) => {
    switch (category) {
      case PermissionCategory.DASHBOARD:
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
      case PermissionCategory.API:
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
      case PermissionCategory.ANALYTICS:
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
      case PermissionCategory.ADMIN:
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300';
      case PermissionCategory.BILLING:
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300';
      case PermissionCategory.DATA:
        return 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900/20 dark:text-indigo-300';
      case PermissionCategory.SUPPORT:
        return 'bg-pink-100 text-pink-800 dark:bg-pink-900/20 dark:text-pink-300';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
    }
  };

  const getTemplateCategoryColor = (category: string) => {
    switch (category) {
      case 'User':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
      case 'Admin':
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
      case 'Support':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
      case 'API':
        return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
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

  const getScopeLabel = (scope: PermissionScope) => {
    switch (scope) {
      case PermissionScope.OWN:
        return 'Personal';
      case PermissionScope.COMPANY:
        return 'Company';
      case PermissionScope.PARTNER:
        return 'Partner';
      case PermissionScope.GLOBAL:
        return 'Global';
      default:
        return 'Unknown';
    }
  };

  const permissionsByCategory = template.permissions.reduce((acc, permissionId) => {
    const permission = AVAILABLE_PERMISSIONS[permissionId];
    if (permission) {
      if (!acc[permission.category]) {
        acc[permission.category] = [];
      }
      acc[permission.category].push(permission);
    }
    return acc;
  }, {} as Record<PermissionCategory, Permission[]>);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-3xl w-full max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
          <div className="flex-1">
            <div className="flex items-center gap-3 mb-2">
              <Shield className="h-6 w-6 text-blue-600" />
              <h3 className="text-xl font-semibold text-gray-900 dark:text-white">
                {template.name}
              </h3>
              {template.isSystem && (
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300">
                  System Template
                </span>
              )}
              <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${getTemplateCategoryColor(template.category)}`}>
                {template.category}
              </span>
            </div>
            <p className="text-gray-600 dark:text-gray-400">{template.description}</p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 ml-4"
          >
            <X className="h-6 w-6" />
          </button>
        </div>

        <div className="overflow-y-auto max-h-[calc(90vh-180px)]">
          <div className="p-6 space-y-6">
            {/* Template Info */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <div className="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-1">
                  <Users className="h-4 w-4 mr-2" />
                  Usage
                </div>
                <div className="text-2xl font-semibold text-gray-900 dark:text-white">
                  {template.usageCount}
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">users assigned</div>
              </div>

              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <div className="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-1">
                  <Shield className="h-4 w-4 mr-2" />
                  Permissions
                </div>
                <div className="text-2xl font-semibold text-gray-900 dark:text-white">
                  {template.permissions.length}
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">total permissions</div>
              </div>

              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <div className="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-1">
                  <Calendar className="h-4 w-4 mr-2" />
                  Updated
                </div>
                <div className="text-sm font-medium text-gray-900 dark:text-white">
                  {template.updatedAt.toLocaleDateString()}
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  Created {template.createdAt.toLocaleDateString()}
                </div>
              </div>
            </div>

            {/* Tags */}
            {template.tags.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center">
                  <Tag className="h-4 w-4 mr-2" />
                  Tags
                </h4>
                <div className="flex flex-wrap gap-2">
                  {template.tags.map((tag, index) => (
                    <span
                      key={index}
                      className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Permissions by Category */}
            <div>
              <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-4 flex items-center">
                <Shield className="h-4 w-4 mr-2" />
                Permissions ({template.permissions.length})
              </h4>
              
              {Object.entries(permissionsByCategory).map(([category, permissions]) => (
                <div key={category} className="mb-6 last:mb-0">
                  <div className="flex items-center gap-2 mb-3">
                    <h5 className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${getCategoryColor(category as PermissionCategory)}`}>
                      {category.charAt(0).toUpperCase() + category.slice(1)}
                    </h5>
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                      {permissions.length} permission{permissions.length !== 1 ? 's' : ''}
                    </span>
                  </div>
                  
                  <div className="space-y-2">
                    {permissions.map((permission) => (
                      <div
                        key={permission.id}
                        className="border border-gray-200 dark:border-gray-600 rounded-lg p-3 bg-gray-50 dark:bg-gray-700"
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-1">
                              <h6 className="font-medium text-gray-900 dark:text-white text-sm">
                                {permission.name}
                              </h6>
                              <span className="text-lg" title={`Scope: ${getScopeLabel(permission.scope)}`}>
                                {getScopeIcon(permission.scope)}
                              </span>
                            </div>
                            <p className="text-xs text-gray-600 dark:text-gray-400 mb-2">
                              {permission.description}
                            </p>
                            <div className="flex items-center gap-2">
                              <span className="text-xs text-gray-500 dark:text-gray-400 font-mono bg-white dark:bg-gray-800 px-2 py-1 rounded">
                                {permission.action}:{permission.resource}
                              </span>
                              <span className="text-xs text-gray-400">
                                {getScopeLabel(permission.scope)} scope
                              </span>
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}

              {template.permissions.length === 0 && (
                <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                  <Shield className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p>No permissions defined for this template</p>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t border-gray-200 dark:border-gray-700">
          <div className="text-sm text-gray-500 dark:text-gray-400">
            Template ID: <span className="font-mono">{template.id}</span>
          </div>
          
          <div className="flex gap-3">
            {onDuplicate && (
              <button
                onClick={() => onDuplicate(template)}
                className="inline-flex items-center px-3 py-2 border border-gray-300 dark:border-gray-600 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                <Copy className="h-4 w-4 mr-2" />
                Duplicate
              </button>
            )}
            
            {onEdit && !template.isSystem && (
              <button
                onClick={() => onEdit(template)}
                className="inline-flex items-center px-3 py-2 border border-gray-300 dark:border-gray-600 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                <Edit className="h-4 w-4 mr-2" />
                Edit
              </button>
            )}

            {onDelete && !template.isSystem && (
              <button
                onClick={() => onDelete(template)}
                className="inline-flex items-center px-3 py-2 border border-red-300 dark:border-red-600 shadow-sm text-sm leading-4 font-medium rounded-md text-red-700 dark:text-red-300 bg-white dark:bg-gray-700 hover:bg-red-50 dark:hover:bg-red-900/20 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
              >
                <AlertTriangle className="h-4 w-4 mr-2" />
                Delete
              </button>
            )}
            
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              <Eye className="h-4 w-4 mr-2 inline" />
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
