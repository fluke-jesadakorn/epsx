'use client';

import { HeadphonesIcon, Settings, Shield, Users, X } from 'lucide-react';
import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
import { Button } from '../../ui/form-components';

interface CreateTemplateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (templateData: TemplateData) => void;
}

interface TemplateData {
  name: string;
  description: string;
  category: 'User' | 'Admin' | 'Support' | 'Manager';
  permissions: string[];
}

export const CreateTemplateModal: React.FC<CreateTemplateModalProps> = ({
  isOpen,
  onClose,
  onSave,
}) => {
  const [formData, setFormData] = useState<TemplateData>({
    name: '',
    description: '',
    category: 'User',
    permissions: [],
  });

  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      await onSave(formData);
      setFormData({
        name: '',
        description: '',
        category: 'User',
        permissions: [],
      });
      onClose();
    } catch (error) {
      console.error('Error creating template:', error);
    } finally {
      setLoading(false);
    }
  };

  const categories = [
    { value: 'User', label: 'User', icon: <Users className="h-4 w-4" /> },
    { value: 'Admin', label: 'Admin', icon: <Shield className="h-4 w-4" /> },
    {
      value: 'Support',
      label: 'Support',
      icon: <HeadphonesIcon className="h-4 w-4" />,
    },
    {
      value: 'Manager',
      label: 'Manager',
      icon: <Settings className="h-4 w-4" />,
    },
  ];

  const availablePermissions = [
    {
      id: 'read:users',
      name: 'Read Users',
      description: 'View user information',
    },
    {
      id: 'write:users',
      name: 'Write Users',
      description: 'Create and edit users',
    },
    { id: 'delete:users', name: 'Delete Users', description: 'Remove users' },
    {
      id: 'read:analytics',
      name: 'Read Analytics',
      description: 'View analytics data',
    },
    {
      id: 'write:analytics',
      name: 'Write Analytics',
      description: 'Modify analytics settings',
    },
    {
      id: 'admin:settings',
      name: 'Admin Settings',
      description: 'Access admin configuration',
    },
    {
      id: 'support:tickets',
      name: 'Support Tickets',
      description: 'Manage support tickets',
    },
    {
      id: 'manage:billing',
      name: 'Manage Billing',
      description: 'Handle billing operations',
    },
  ];

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <Card className="w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Create Permission Template
          </CardTitle>
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            className="h-6 w-6"
          >
            <X className="h-4 w-4" />
          </Button>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            <div>
              <label className="block text-sm font-medium mb-1">
                Template Name
              </label>
              <input
                type="text"
                required
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                placeholder="e.g., Standard User Access"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">
                Description
              </label>
              <textarea
                required
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                value={formData.description}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
                placeholder="Describe what this template is for..."
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Category</label>
              <div className="grid grid-cols-2 gap-2">
                {categories.map((category) => (
                  <button
                    key={category.value}
                    type="button"
                    className={`p-3 border rounded-md flex items-center gap-2 transition-colors ${
                      formData.category === category.value
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-300 hover:bg-gray-50'
                    }`}
                    onClick={() =>
                      setFormData({
                        ...formData,
                        category: category.value as any,
                      })
                    }
                  >
                    {category.icon}
                    <span className="text-sm">{category.label}</span>
                  </button>
                ))}
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Permissions
              </label>
              <div className="space-y-2 max-h-60 overflow-y-auto border border-gray-200 rounded-md p-3">
                {availablePermissions.map((permission) => (
                  <label
                    key={permission.id}
                    className="flex items-start gap-3 p-2 hover:bg-gray-50 rounded"
                  >
                    <input
                      type="checkbox"
                      className="mt-1"
                      checked={formData.permissions.includes(permission.id)}
                      onChange={(e) => {
                        if (e.target.checked) {
                          setFormData({
                            ...formData,
                            permissions: [
                              ...formData.permissions,
                              permission.id,
                            ],
                          });
                        } else {
                          setFormData({
                            ...formData,
                            permissions: formData.permissions.filter(
                              (p) => p !== permission.id,
                            ),
                          });
                        }
                      }}
                    />
                    <div className="flex-1">
                      <div className="text-sm font-medium">
                        {permission.name}
                      </div>
                      <div className="text-xs text-gray-500">
                        {permission.description}
                      </div>
                    </div>
                  </label>
                ))}
              </div>
              <div className="text-xs text-gray-500 mt-1">
                {formData.permissions.length} permission
                {formData.permissions.length !== 1 ? 's' : ''} selected
              </div>
            </div>

            <div className="flex gap-2 pt-4">
              <Button
                type="button"
                variant="outline"
                onClick={onClose}
                className="flex-1"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={loading || formData.permissions.length === 0}
                className="flex-1"
              >
                {loading ? 'Creating...' : 'Create Template'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};
