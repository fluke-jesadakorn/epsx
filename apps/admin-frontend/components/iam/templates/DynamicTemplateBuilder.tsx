'use client';

import React, { useState, useEffect } from 'react';
import { 
  Shield, 
  X, 
  Plus, 
  Trash2, 
  AlertTriangle, 
  CheckCircle, 
  Info,
  Eye,
  Settings,
  Users,
  Database,
  BarChart3,
  CreditCard,
  Headphones
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '../../ui/form-components';
import {
  DynamicTemplate,
  TemplatePermission,
  PermissionCategory,
  PermissionScope,
  TemplateScope,
  TemplateStatus,
  PackageTier,
  ConflictResolutionStrategy,
  TemplateValidationResult,
  TemplatePreview,
  PermissionCondition
} from '@epsx/types';

interface DynamicTemplateBuilderProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (template: Omit<DynamicTemplate, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'>) => void;
  editingTemplate?: DynamicTemplate;
  availablePermissions: TemplatePermission[];
}

export const DynamicTemplateBuilder: React.FC<DynamicTemplateBuilderProps> = ({
  isOpen,
  onClose,
  onSave,
  editingTemplate,
  availablePermissions,
}) => {
  const [formData, setFormData] = useState<Partial<DynamicTemplate>>({
    name: '',
    description: '',
    permissions: [],
    scope: TemplateScope.PERSONAL,
    status: TemplateStatus.DRAFT,
    packageTierCompatibility: [],
    validationRules: [],
    conflictResolution: ConflictResolutionStrategy.FAIL,
    categories: [],
    tags: [],
    isPublic: false,
    sharedWith: [],
    inheritanceMode: 'extend',
  });

  const [selectedCategory, setSelectedCategory] = useState<PermissionCategory | 'all'>('all');
  const [validation, setValidation] = useState<TemplateValidationResult | null>(null);
  const [preview, setPreview] = useState<TemplatePreview | null>(null);
  const [showPreview, setShowPreview] = useState(false);
  const [loading, setLoading] = useState(false);

  // Initialize form with editing template data
  useEffect(() => {
    if (editingTemplate) {
      setFormData(editingTemplate);
    }
  }, [editingTemplate]);

  // Category icons mapping
  const categoryIcons = {
    [PermissionCategory.DASHBOARD]: <BarChart3 className="h-4 w-4" />,
    [PermissionCategory.API]: <Database className="h-4 w-4" />,
    [PermissionCategory.DATA]: <Database className="h-4 w-4" />,
    [PermissionCategory.ADMIN]: <Shield className="h-4 w-4" />,
    [PermissionCategory.ANALYTICS]: <BarChart3 className="h-4 w-4" />,
    [PermissionCategory.INTEGRATION]: <Settings className="h-4 w-4" />,
    [PermissionCategory.BILLING]: <CreditCard className="h-4 w-4" />,
    [PermissionCategory.SUPPORT]: <Headphones className="h-4 w-4" />,
  };

  // Filter permissions by category
  const filteredPermissions = selectedCategory === 'all' 
    ? availablePermissions
    : availablePermissions.filter(p => p.category === selectedCategory);

  // Handle permission selection
  const handlePermissionToggle = (permission: TemplatePermission) => {
    const currentPermissions = formData.permissions || [];
    const isSelected = currentPermissions.some(p => p.id === permission.id);

    if (isSelected) {
      setFormData({
        ...formData,
        permissions: currentPermissions.filter(p => p.id !== permission.id),
      });
    } else {
      setFormData({
        ...formData,
        permissions: [...currentPermissions, permission],
      });
    }
  };

  // Add permission condition
  const addPermissionCondition = (permissionId: string, condition: PermissionCondition) => {
    const updatedPermissions = (formData.permissions || []).map(p => {
      if (p.id === permissionId) {
        return {
          ...p,
          conditions: [...(p.conditions || []), condition],
        };
      }
      return p;
    });

    setFormData({
      ...formData,
      permissions: updatedPermissions,
    });
  };

  // Validate template
  const validateTemplate = async () => {
    // Mock validation logic - in real implementation, this would call a service
    const errors: any[] = [];
    const warnings: any[] = [];
    const info: any[] = [];

    if (!formData.name?.trim()) {
      errors.push({
        code: 'MISSING_NAME',
        message: 'Template name is required',
        field: 'name',
        suggestion: 'Provide a descriptive name for your template',
      });
    }

    if (!formData.permissions?.length) {
      errors.push({
        code: 'NO_PERMISSIONS',
        message: 'At least one permission is required',
        field: 'permissions',
        suggestion: 'Select permissions from the available list',
      });
    }

    if (formData.permissions && formData.permissions.length > 50) {
      warnings.push({
        code: 'TOO_MANY_PERMISSIONS',
        message: 'Template has many permissions which may impact performance',
        field: 'permissions',
        suggestion: 'Consider breaking this into multiple specialized templates',
      });
    }

    const validationResult: TemplateValidationResult = {
      isValid: errors.length === 0,
      errors,
      warnings,
      info,
    };

    setValidation(validationResult);
    return validationResult;
  };

  // Generate preview
  const generatePreview = async () => {
    if (!formData.permissions?.length) return;

    // Mock preview generation - in real implementation, this would call a service
    const preview: TemplatePreview = {
      effectivePermissions: formData.permissions,
      conflicts: [],
      inheritanceChain: formData.parentTemplate ? [formData.parentTemplate] : [],
      packageCompatibility: Object.values(PackageTier).map(tier => ({
        packageTier: tier,
        compatible: true,
        issues: [],
        suggestions: [],
      })),
    };

    setPreview(preview);
  };

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      const validationResult = await validateTemplate();
      
      if (!validationResult.isValid) {
        setLoading(false);
        return;
      }

      const templateData: Omit<DynamicTemplate, 'id' | 'createdAt' | 'updatedAt' | 'usageCount' | 'assignedUserCount'> = {
        name: formData.name!,
        description: formData.description!,
        version: editingTemplate ? editingTemplate.version + 1 : 1,
        permissions: formData.permissions!,
        parentTemplate: formData.parentTemplate,
        inheritanceMode: formData.inheritanceMode!,
        scope: formData.scope!,
        status: formData.status!,
        packageTierCompatibility: formData.packageTierCompatibility!,
        minimumPackageTier: formData.minimumPackageTier,
        validationRules: formData.validationRules!,
        conflictResolution: formData.conflictResolution!,
        createdBy: 'current-user-id', // This would come from auth context
        updatedBy: 'current-user-id',
        categories: formData.categories!,
        tags: formData.tags!,
        isPublic: formData.isPublic!,
        sharedWith: formData.sharedWith!,
      };

      await onSave(templateData);
      onClose();
    } catch (error) {
      console.error('Error saving template:', error);
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <Card className="w-full max-w-6xl mx-4 max-h-[95vh] overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between border-b">
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            {editingTemplate ? 'Edit Template' : 'Create Dynamic Template'}
          </CardTitle>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                generatePreview();
                setShowPreview(!showPreview);
              }}
              className="flex items-center gap-2"
            >
              <Eye className="h-4 w-4" />
              Preview
            </Button>
            <Button
              variant="ghost"
              size="icon"
              onClick={onClose}
              className="h-6 w-6"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </CardHeader>

        <div className="flex h-[calc(95vh-120px)]">
          {/* Main Form */}
          <div className="flex-1 overflow-y-auto p-6">
            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Basic Information */}
              <div className="space-y-4">
                <h3 className="text-lg font-medium">Basic Information</h3>
                
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium mb-1">
                      Template Name *
                    </label>
                    <input
                      type="text"
                      required
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      value={formData.name || ''}
                      onChange={(e) =>
                        setFormData({ ...formData, name: e.target.value })
                      }
                      placeholder="e.g., Marketing Team Access"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-1">
                      Template Scope
                    </label>
                    <select
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      value={formData.scope}
                      onChange={(e) =>
                        setFormData({ ...formData, scope: e.target.value as TemplateScope })
                      }
                    >
                      <option value={TemplateScope.PERSONAL}>Personal</option>
                      <option value={TemplateScope.ORGANIZATION}>Organization</option>
                      <option value={TemplateScope.SYSTEM}>System</option>
                    </select>
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium mb-1">
                    Description *
                  </label>
                  <textarea
                    required
                    rows={3}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    value={formData.description || ''}
                    onChange={(e) =>
                      setFormData({ ...formData, description: e.target.value })
                    }
                    placeholder="Describe what this template is for and who should use it..."
                  />
                </div>

                <div className="grid grid-cols-3 gap-4">
                  <div>
                    <label className="block text-sm font-medium mb-1">
                      Status
                    </label>
                    <select
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      value={formData.status}
                      onChange={(e) =>
                        setFormData({ ...formData, status: e.target.value as TemplateStatus })
                      }
                    >
                      <option value={TemplateStatus.DRAFT}>Draft</option>
                      <option value={TemplateStatus.ACTIVE}>Active</option>
                      <option value={TemplateStatus.ARCHIVED}>Archived</option>
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-1">
                      Conflict Resolution
                    </label>
                    <select
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      value={formData.conflictResolution}
                      onChange={(e) =>
                        setFormData({ ...formData, conflictResolution: e.target.value as ConflictResolutionStrategy })
                      }
                    >
                      <option value={ConflictResolutionStrategy.FAIL}>Fail on Conflict</option>
                      <option value={ConflictResolutionStrategy.MERGE_PERMISSIVE}>Merge Permissive</option>
                      <option value={ConflictResolutionStrategy.MERGE_RESTRICTIVE}>Merge Restrictive</option>
                    </select>
                  </div>

                  <div className="flex items-center gap-2 pt-6">
                    <input
                      type="checkbox"
                      id="isPublic"
                      checked={formData.isPublic}
                      onChange={(e) =>
                        setFormData({ ...formData, isPublic: e.target.checked })
                      }
                    />
                    <label htmlFor="isPublic" className="text-sm font-medium">
                      Public Template
                    </label>
                  </div>
                </div>
              </div>

              {/* Permission Selection */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-medium">Permissions</h3>
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-gray-500">Filter by category:</span>
                    <select
                      className="px-2 py-1 border border-gray-300 rounded text-sm"
                      value={selectedCategory}
                      onChange={(e) => setSelectedCategory(e.target.value as PermissionCategory | 'all')}
                    >
                      <option value="all">All Categories</option>
                      {Object.values(PermissionCategory).map(category => (
                        <option key={category} value={category}>
                          {category.charAt(0).toUpperCase() + category.slice(1)}
                        </option>
                      ))}
                    </select>
                  </div>
                </div>

                <div className="border border-gray-200 rounded-md max-h-80 overflow-y-auto">
                  <div className="p-4 space-y-2">
                    {filteredPermissions.map((permission) => {
                      const isSelected = (formData.permissions || []).some(p => p.id === permission.id);
                      
                      return (
                        <label
                          key={permission.id}
                          className={`flex items-start gap-3 p-3 rounded border cursor-pointer transition-colors ${
                            isSelected
                              ? 'border-blue-500 bg-blue-50'
                              : 'border-gray-200 hover:bg-gray-50'
                          }`}
                        >
                          <input
                            type="checkbox"
                            className="mt-1"
                            checked={isSelected}
                            onChange={() => handlePermissionToggle(permission)}
                          />
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-1">
                              {categoryIcons[permission.category]}
                              <span className="font-medium text-sm">{permission.name}</span>
                              <span className="text-xs bg-gray-100 px-2 py-0.5 rounded">
                                {permission.scope}
                              </span>
                            </div>
                            <p className="text-xs text-gray-600 mb-1">{permission.description}</p>
                            <div className="text-xs text-gray-500">
                              <span className="font-mono bg-gray-100 px-1 rounded">
                                {permission.id}
                              </span>
                            </div>
                          </div>
                        </label>
                      );
                    })}
                  </div>
                </div>

                <div className="text-sm text-gray-500">
                  {(formData.permissions || []).length} permission(s) selected
                </div>
              </div>

              {/* Package Compatibility */}
              <div className="space-y-4">
                <h3 className="text-lg font-medium">Package Compatibility</h3>
                <div className="grid grid-cols-3 gap-2">
                  {Object.values(PackageTier).map(tier => (
                    <label
                      key={tier}
                      className="flex items-center gap-2 p-2 border rounded cursor-pointer hover:bg-gray-50"
                    >
                      <input
                        type="checkbox"
                        checked={(formData.packageTierCompatibility || []).includes(tier)}
                        onChange={(e) => {
                          const current = formData.packageTierCompatibility || [];
                          if (e.target.checked) {
                            setFormData({
                              ...formData,
                              packageTierCompatibility: [...current, tier],
                            });
                          } else {
                            setFormData({
                              ...formData,
                              packageTierCompatibility: current.filter(t => t !== tier),
                            });
                          }
                        }}
                      />
                      <span className="text-sm font-medium">{tier}</span>
                    </label>
                  ))}
                </div>
              </div>

              {/* Actions */}
              <div className="flex gap-2 pt-6 border-t">
                <Button
                  type="button"
                  variant="outline"
                  onClick={onClose}
                  className="flex-1"
                >
                  Cancel
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={validateTemplate}
                  className="flex-1"
                >
                  Validate
                </Button>
                <Button
                  type="submit"
                  disabled={loading || !(formData.permissions?.length)}
                  className="flex-1"
                >
                  {loading ? 'Saving...' : editingTemplate ? 'Update Template' : 'Create Template'}
                </Button>
              </div>
            </form>
          </div>

          {/* Preview/Validation Sidebar */}
          {(showPreview || validation) && (
            <div className="w-80 border-l bg-gray-50 overflow-y-auto">
              <div className="p-4 space-y-4">
                {validation && (
                  <Card>
                    <CardHeader className="pb-2">
                      <CardTitle className="text-sm flex items-center gap-2">
                        {validation.isValid ? (
                          <CheckCircle className="h-4 w-4 text-green-500" />
                        ) : (
                          <AlertTriangle className="h-4 w-4 text-red-500" />
                        )}
                        Validation Results
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-2">
                      {validation.errors.map((error, index) => (
                        <div key={index} className="text-xs p-2 bg-red-50 border border-red-200 rounded">
                          <div className="font-medium text-red-800">{error.message}</div>
                          {error.suggestion && (
                            <div className="text-red-600 mt-1">{error.suggestion}</div>
                          )}
                        </div>
                      ))}
                      {validation.warnings.map((warning, index) => (
                        <div key={index} className="text-xs p-2 bg-yellow-50 border border-yellow-200 rounded">
                          <div className="font-medium text-yellow-800">{warning.message}</div>
                          {warning.suggestion && (
                            <div className="text-yellow-600 mt-1">{warning.suggestion}</div>
                          )}
                        </div>
                      ))}
                      {validation.isValid && (
                        <div className="text-xs p-2 bg-green-50 border border-green-200 rounded text-green-800">
                          Template validation passed!
                        </div>
                      )}
                    </CardContent>
                  </Card>
                )}

                {showPreview && preview && (
                  <Card>
                    <CardHeader className="pb-2">
                      <CardTitle className="text-sm flex items-center gap-2">
                        <Eye className="h-4 w-4" />
                        Template Preview
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div>
                        <div className="text-xs font-medium mb-1">Effective Permissions ({preview.effectivePermissions.length})</div>
                        <div className="max-h-32 overflow-y-auto space-y-1">
                          {preview.effectivePermissions.map(permission => (
                            <div key={permission.id} className="text-xs p-1 bg-blue-50 rounded">
                              {permission.name}
                            </div>
                          ))}
                        </div>
                      </div>

                      {preview.conflicts.length > 0 && (
                        <div>
                          <div className="text-xs font-medium mb-1 text-red-600">Conflicts Detected</div>
                          {preview.conflicts.map((conflict, index) => (
                            <div key={index} className="text-xs p-2 bg-red-50 border border-red-200 rounded">
                              {conflict.description}
                            </div>
                          ))}
                        </div>
                      )}

                      <div>
                        <div className="text-xs font-medium mb-1">Package Compatibility</div>
                        <div className="space-y-1">
                          {preview.packageCompatibility.map(result => (
                            <div
                              key={result.packageTier}
                              className={`text-xs p-1 rounded flex items-center justify-between ${
                                result.compatible ? 'bg-green-50 text-green-800' : 'bg-red-50 text-red-800'
                              }`}
                            >
                              <span>{result.packageTier}</span>
                              <span>{result.compatible ? '✓' : '✗'}</span>
                            </div>
                          ))}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                )}
              </div>
            </div>
          )}
        </div>
      </Card>
    </div>
  );
};