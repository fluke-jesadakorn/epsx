'use client';

import { Mail, Package, Shield, User, X } from 'lucide-react';
import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
import { Button } from '../../ui/form-components';
import { useToast } from '../../ui/toast';

interface CreateUserModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (userData: UserData) => void;
}

interface UserData {
  name: string;
  email: string;
  packageTier: string;
  permissions: string[];
}

export const CreateUserModal: React.FC<CreateUserModalProps> = ({
  isOpen,
  onClose,
  onSave,
}) => {
  const [formData, setFormData] = useState<UserData>({
    name: '',
    email: '',
    packageTier: 'free',
    permissions: [],
  });

  const [loading, setLoading] = useState(false);
  const { addToast } = useToast();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      await onSave(formData);
      setFormData({
        name: '',
        email: '',
        packageTier: 'free',
        permissions: [],
      });
      addToast({
        type: 'success',
        title: 'User Created',
        description: `Successfully created user ${formData.name}`,
      });
      onClose();
    } catch (error) {
      console.error('Error creating user:', error);
      addToast({
        type: 'error',
        title: 'Failed to Create User',
        description: 'There was an error creating the user. Please try again.',
      });
    } finally {
      setLoading(false);
    }
  };

  const packageTiers = [
    { value: 'free', label: 'Free Tier' },
    { value: 'premium', label: 'Premium' },
    { value: 'enterprise', label: 'Enterprise' },
  ];

  const availablePermissions = [
    'read:users',
    'write:users',
    'read:analytics',
    'write:analytics',
    'admin:settings',
  ];

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <Card className="w-full max-w-md mx-4">
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <User className="h-5 w-5" />
            Create New User
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
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-1">
                <Mail className="h-4 w-4 inline mr-2" />
                Email
              </label>
              <input
                type="email"
                required
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                value={formData.email}
                onChange={(e) =>
                  setFormData({ ...formData, email: e.target.value })
                }
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">
                <User className="h-4 w-4 inline mr-2" />
                Display Name
              </label>
              <input
                type="text"
                required
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">
                <Package className="h-4 w-4 inline mr-2" />
                Package Tier
              </label>
              <select
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                value={formData.packageTier}
                onChange={(e) =>
                  setFormData({ ...formData, packageTier: e.target.value })
                }
              >
                {packageTiers.map((tier) => (
                  <option key={tier.value} value={tier.value}>
                    {tier.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                <Shield className="h-4 w-4 inline mr-2" />
                Additional Permissions
              </label>
              <div className="space-y-2 max-h-32 overflow-y-auto">
                {availablePermissions.map((permission) => (
                  <label key={permission} className="flex items-center">
                    <input
                      type="checkbox"
                      className="mr-2"
                      checked={formData.permissions.includes(permission)}
                      onChange={(e) => {
                        if (e.target.checked) {
                          setFormData({
                            ...formData,
                            permissions: [...formData.permissions, permission],
                          });
                        } else {
                          setFormData({
                            ...formData,
                            permissions: formData.permissions.filter(
                              (p) => p !== permission,
                            ),
                          });
                        }
                      }}
                    />
                    <span className="text-sm">{permission}</span>
                  </label>
                ))}
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
              <Button type="submit" disabled={loading} className="flex-1">
                {loading ? 'Creating...' : 'Create User'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};
