'use client';

import React from 'react';
import { Badge, Button } from '../../ui/form-components';
import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
import { Shield, Users, Check, X } from 'lucide-react';

interface TemplatePreviewModalProps {
  template: {
    id: string;
    name: string;
    description: string;
    category: string;
    permissions: string[];
    usageCount: number;
    isActive: boolean;
  };
  open: boolean;
  onClose: () => void;
}

export const TemplatePreviewModal: React.FC<TemplatePreviewModalProps> = ({ 
  template, 
  open, 
  onClose 
}) => {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black bg-opacity-50" onClick={onClose} />
      <div className="relative z-50 w-full max-w-2xl bg-white rounded-lg shadow-xl">
        <div className="flex items-center justify-between p-6 border-b">
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            <h2 className="text-xl font-semibold">{template.name}</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-100 rounded-md"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Template Information</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-sm font-medium text-gray-500">Category</label>
                  <Badge variant="outline" className="block w-fit mt-1">{template.category}</Badge>
                </div>
                <div>
                  <label className="text-sm font-medium text-gray-500">Status</label>
                  <Badge variant={template.isActive ? 'default' : 'secondary'} className="block w-fit mt-1">
                    {template.isActive ? 'Active' : 'Inactive'}
                  </Badge>
                </div>
                <div>
                  <label className="text-sm font-medium text-gray-500">Users Assigned</label>
                  <div className="flex items-center gap-1 mt-1">
                    <Users className="h-4 w-4 text-gray-500" />
                    <span className="text-sm">{template.usageCount}</span>
                  </div>
                </div>
                <div>
                  <label className="text-sm font-medium text-gray-500">Permissions Count</label>
                  <span className="text-sm block mt-1">{template.permissions.length}</span>
                </div>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Description</label>
                <p className="text-sm mt-1">{template.description}</p>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Included Permissions</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 gap-2 max-h-60 overflow-y-auto">
                {template.permissions.map((permission, index) => (
                  <div key={index} className="flex items-center gap-2 p-2 border rounded-lg">
                    <Check className="h-4 w-4 text-green-500 flex-shrink-0" />
                    <span className="text-sm">{permission}</span>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="flex justify-end gap-3 p-6 border-t">
          <Button variant="outline" onClick={onClose}>Close</Button>
          <Button>Apply to User</Button>
        </div>
      </div>
    </div>
  );
};
