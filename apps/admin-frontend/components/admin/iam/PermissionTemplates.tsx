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
  AlertTriangle
} from 'lucide-react';

interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  permissions: Array<{
    action: string;
    resource: string;
  }>;
  usageCount: number;
  isSystem: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export const PermissionTemplates: React.FC = () => {
  const [templates] = useState<PermissionTemplate[]>([
    {
      id: '1',
      name: 'Basic User Access',
      description: 'Standard permissions for regular users',
      category: 'User',
      permissions: [
        { action: 'read', resource: 'dashboard' },
        { action: 'read', resource: 'profile' },
        { action: 'update', resource: 'profile' }
      ],
      usageCount: 156,
      isSystem: true,
      createdAt: new Date('2024-01-15'),
      updatedAt: new Date('2024-03-10')
    },
    {
      id: '2',
      name: 'Premium User Access',
      description: 'Enhanced permissions for premium users',
      category: 'User',
      permissions: [
        { action: 'read', resource: 'dashboard' },
        { action: 'read', resource: 'analytics' },
        { action: 'read', resource: 'api_access' },
        { action: 'update', resource: 'profile' }
      ],
      usageCount: 89,
      isSystem: true,
      createdAt: new Date('2024-01-15'),
      updatedAt: new Date('2024-03-10')
    },
    {
      id: '3',
      name: 'Support Agent',
      description: 'Permissions for customer support team',
      category: 'Admin',
      permissions: [
        { action: 'read', resource: 'users' },
        { action: 'update', resource: 'user_support' },
        { action: 'read', resource: 'tickets' }
      ],
      usageCount: 12,
      isSystem: false,
      createdAt: new Date('2024-02-01'),
      updatedAt: new Date('2024-03-15')
    }
  ]);
  
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('');
  const [showCreateModal, setShowCreateModal] = useState(false);

  const categories = ['All Categories', 'User', 'Admin', 'Support', 'API'];
  
  const filteredTemplates = templates.filter(template => {
    const matchesSearch = !searchQuery || 
      template.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.description.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesCategory = !selectedCategory || 
      selectedCategory === 'All Categories' || 
      template.category === selectedCategory;
    
    return matchesSearch && matchesCategory;
  });

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'User':
        return 'bg-blue-100 text-blue-800';
      case 'Admin':
        return 'bg-purple-100 text-purple-800';
      case 'Support':
        return 'bg-green-100 text-green-800';
      case 'API':
        return 'bg-orange-100 text-orange-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row gap-4 justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900">Permission Templates</h3>
          <p className="text-sm text-gray-500 mt-1">
            Create and manage reusable permission templates for quick user setup
          </p>
        </div>
        
        <button
          onClick={() => setShowCreateModal(true)}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700"
        >
          <Plus className="h-4 w-4 mr-2" />
          Create Template
        </button>
      </div>

      {/* Search and Filter */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
          <input
            type="text"
            placeholder="Search templates..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        
        <select
          value={selectedCategory}
          onChange={(e) => setSelectedCategory(e.target.value)}
          className="border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        >
          {categories.map((category) => (
            <option key={category} value={category === 'All Categories' ? '' : category}>
              {category}
            </option>
          ))}
        </select>
      </div>

      {/* Templates Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredTemplates.map((template) => (
          <div key={template.id} className="bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow">
            <div className="p-6">
              <div className="flex items-start justify-between mb-4">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <h4 className="font-medium text-gray-900">{template.name}</h4>
                    {template.isSystem && (
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                        System
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500 mb-3">{template.description}</p>
                  
                  <div className="flex items-center gap-2 mb-3">
                    <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${getCategoryColor(template.category)}`}>
                      {template.category}
                    </span>
                    <span className="inline-flex items-center text-xs text-gray-500">
                      <Users className="h-3 w-3 mr-1" />
                      {template.usageCount} users
                    </span>
                  </div>
                </div>
                
                <div className="relative">
                  <button className="text-gray-400 hover:text-gray-600">
                    <MoreHorizontal className="h-5 w-5" />
                  </button>
                </div>
              </div>

              {/* Permissions Preview */}
              <div className="mb-4">
                <h5 className="text-xs font-medium text-gray-700 mb-2 flex items-center">
                  <Shield className="h-3 w-3 mr-1" />
                  Permissions ({template.permissions.length})
                </h5>
                <div className="space-y-1">
                  {template.permissions.slice(0, 3).map((permission, index) => (
                    <div key={index} className="text-xs text-gray-500 flex items-center">
                      <span className="w-2 h-2 bg-blue-400 rounded-full mr-2"></span>
                      <span className="font-mono">{permission.action}</span>
                      <span className="mx-1">on</span>
                      <span className="font-mono">{permission.resource}</span>
                    </div>
                  ))}
                  {template.permissions.length > 3 && (
                    <div className="text-xs text-gray-400">
                      +{template.permissions.length - 3} more...
                    </div>
                  )}
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center justify-between pt-4 border-t border-gray-100">
                <div className="text-xs text-gray-500">
                  Updated {template.updatedAt.toLocaleDateString()}
                </div>
                
                <div className="flex items-center gap-2">
                  <button className="text-gray-400 hover:text-blue-600 transition-colors">
                    <Eye className="h-4 w-4" />
                  </button>
                  <button className="text-gray-400 hover:text-green-600 transition-colors">
                    <Copy className="h-4 w-4" />
                  </button>
                  {!template.isSystem && (
                    <>
                      <button className="text-gray-400 hover:text-blue-600 transition-colors">
                        <Edit className="h-4 w-4" />
                      </button>
                      <button className="text-gray-400 hover:text-red-600 transition-colors">
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

      {filteredTemplates.length === 0 && (
        <div className="text-center py-12">
          <Shield className="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">No templates found</h3>
          <p className="text-gray-500 mb-4">
            {searchQuery || selectedCategory 
              ? 'Try adjusting your search or filters' 
              : 'Create your first permission template to get started'
            }
          </p>
          {!searchQuery && !selectedCategory && (
            <button
              onClick={() => setShowCreateModal(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700"
            >
              <Plus className="h-4 w-4 mr-2" />
              Create Template
            </button>
          )}
        </div>
      )}

      {/* Create Template Modal Placeholder */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-gray-900">Create Template</h3>
              <button
                onClick={() => setShowCreateModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                ×
              </button>
            </div>
            
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
              <div className="flex">
                <AlertTriangle className="h-5 w-5 text-yellow-400 mr-2 mt-0.5" />
                <div>
                  <h4 className="text-sm font-medium text-yellow-800">Not Implemented</h4>
                  <p className="text-sm text-yellow-700 mt-1">
                    Template creation functionality will be implemented in the next iteration.
                  </p>
                </div>
              </div>
            </div>
            
            <div className="mt-6 flex justify-end">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
