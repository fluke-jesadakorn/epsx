'use client';

import React, { useState, useEffect } from 'react';
import {
  Shield,
  X,
  Search,
  Plus,
  Trash2,
  Calendar,
  AlertTriangle,
  CheckCircle,
  Info,
  Users,
  Eye,
  Clock,
  FileText
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '../../ui/form-components';
import {
  DynamicTemplate,
  TemplateAssignment,
  TemplateScope,
  TemplateStatus,
  PackageTier,
} from '@epsx/types';

interface TemplateAssignmentModalProps {
  isOpen: boolean;
  onClose: () => void;
  userId: string;
  userName: string;
  userPackageTier: PackageTier;
  currentAssignments: TemplateAssignment[];
  availableTemplates: DynamicTemplate[];
  onAssignTemplate: (templateId: string, options?: { expiresAt?: Date; notes?: string }) => Promise<void>;
  onUnassignTemplate: (templateId: string) => Promise<void>;
  onBulkAssign?: (templateIds: string[]) => Promise<void>;
}

export const TemplateAssignmentModal: React.FC<TemplateAssignmentModalProps> = ({
  isOpen,
  onClose,
  userId,
  userName,
  userPackageTier,
  currentAssignments,
  availableTemplates,
  onAssignTemplate,
  onUnassignTemplate,
  onBulkAssign,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTemplates, setSelectedTemplates] = useState<string[]>([]);
  const [showAssignmentForm, setShowAssignmentForm] = useState(false);
  const [assignmentOptions, setAssignmentOptions] = useState({
    templateId: '',
    expiresAt: '',
    notes: '',
  });
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'available' | 'assigned'>('available');

  // Get assigned template IDs for quick lookup
  const assignedTemplateIds = new Set(currentAssignments.map(a => a.templateId));

  // Filter available templates
  const filteredAvailableTemplates = availableTemplates.filter(template => {
    if (assignedTemplateIds.has(template.id)) return false;
    
    const matchesSearch = template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         template.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         template.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
    
    const isCompatible = template.packageTierCompatibility.includes(userPackageTier);
    const isActive = template.status === TemplateStatus.ACTIVE;
    
    return matchesSearch && isCompatible && isActive;
  });

  // Get assigned templates with details
  const assignedTemplatesWithDetails = currentAssignments.map(assignment => {
    const template = availableTemplates.find(t => t.id === assignment.templateId);
    return {
      assignment,
      template,
    };
  }).filter(item => item.template); // Only show assignments where we have template details

  // Handle individual template assignment
  const handleAssignTemplate = async (templateId: string) => {
    if (showAssignmentForm && assignmentOptions.templateId === templateId) {
      // Process the assignment with options
      setLoading(true);
      try {
        const options: { expiresAt?: Date; notes?: string } = {};
        
        if (assignmentOptions.expiresAt) {
          options.expiresAt = new Date(assignmentOptions.expiresAt);
        }
        
        if (assignmentOptions.notes.trim()) {
          options.notes = assignmentOptions.notes.trim();
        }

        await onAssignTemplate(templateId, options);
        
        // Reset form
        setAssignmentOptions({ templateId: '', expiresAt: '', notes: '' });
        setShowAssignmentForm(false);
      } catch (error) {
        console.error('Error assigning template:', error);
      } finally {
        setLoading(false);
      }
    } else {
      // Show assignment form
      setAssignmentOptions({ templateId, expiresAt: '', notes: '' });
      setShowAssignmentForm(true);
    }
  };

  // Handle bulk assignment
  const handleBulkAssignment = async () => {
    if (selectedTemplates.length === 0 || !onBulkAssign) return;
    
    setLoading(true);
    try {
      await onBulkAssign(selectedTemplates);
      setSelectedTemplates([]);
    } catch (error) {
      console.error('Error bulk assigning templates:', error);
    } finally {
      setLoading(false);
    }
  };

  // Handle template unassignment
  const handleUnassignTemplate = async (templateId: string) => {
    setLoading(true);
    try {
      await onUnassignTemplate(templateId);
    } catch (error) {
      console.error('Error unassigning template:', error);
    } finally {
      setLoading(false);
    }
  };

  // Get template compatibility status
  const getCompatibilityStatus = (template: DynamicTemplate) => {
    const isCompatible = template.packageTierCompatibility.includes(userPackageTier);
    const isActive = template.status === TemplateStatus.ACTIVE;
    
    if (!isActive) {
      return { 
        status: 'inactive', 
        message: 'Template is not active', 
        color: 'text-gray-500',
        icon: <AlertTriangle className="h-4 w-4" />
      };
    }
    
    if (!isCompatible) {
      return { 
        status: 'incompatible', 
        message: `Not compatible with ${userPackageTier} tier`, 
        color: 'text-red-500',
        icon: <AlertTriangle className="h-4 w-4" />
      };
    }
    
    return { 
      status: 'compatible', 
      message: 'Compatible with user package', 
      color: 'text-green-500',
      icon: <CheckCircle className="h-4 w-4" />
    };
  };

  // Reset form when modal closes
  useEffect(() => {
    if (!isOpen) {
      setSearchTerm('');
      setSelectedTemplates([]);
      setShowAssignmentForm(false);
      setAssignmentOptions({ templateId: '', expiresAt: '', notes: '' });
      setActiveTab('available');
    }
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <Card className="w-full max-w-6xl mx-4 max-h-[90vh] overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between border-b">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Template Assignments for {userName}
            </CardTitle>
            <p className="text-sm text-gray-600 mt-1">
              Manage dynamic template assignments • Package: {userPackageTier}
            </p>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            className="h-6 w-6"
          >
            <X className="h-4 w-4" />
          </Button>
        </CardHeader>

        <div className="p-6">
          {/* Tab Navigation */}
          <div className="flex border-b mb-6">
            <button
              className={`px-4 py-2 border-b-2 font-medium text-sm ${
                activeTab === 'available'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
              onClick={() => setActiveTab('available')}
            >
              Available Templates ({filteredAvailableTemplates.length})
            </button>
            <button
              className={`px-4 py-2 border-b-2 font-medium text-sm ${
                activeTab === 'assigned'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
              onClick={() => setActiveTab('assigned')}
            >
              Assigned Templates ({currentAssignments.length})
            </button>
          </div>

          {/* Available Templates Tab */}
          {activeTab === 'available' && (
            <div className="space-y-4">
              {/* Search and Actions */}
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
                {selectedTemplates.length > 0 && onBulkAssign && (
                  <Button
                    onClick={handleBulkAssignment}
                    disabled={loading}
                    className="flex items-center gap-2"
                  >
                    <Plus className="h-4 w-4" />
                    Assign Selected ({selectedTemplates.length})
                  </Button>
                )}
              </div>

              {/* Templates List */}
              <div className="max-h-96 overflow-y-auto space-y-3">
                {filteredAvailableTemplates.length === 0 ? (
                  <div className="text-center py-8">
                    <Shield className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">No templates available</h3>
                    <p className="text-gray-600">
                      {searchTerm
                        ? 'No templates match your search criteria'
                        : 'No compatible templates found for this user'}
                    </p>
                  </div>
                ) : (
                  filteredAvailableTemplates.map((template) => {
                    const compatibility = getCompatibilityStatus(template);
                    const isSelected = selectedTemplates.includes(template.id);
                    const showingForm = showAssignmentForm && assignmentOptions.templateId === template.id;

                    return (
                      <Card
                        key={template.id}
                        className={`transition-all ${
                          isSelected ? 'ring-2 ring-blue-500 bg-blue-50' : 'hover:shadow-md'
                        }`}
                      >
                        <CardContent className="p-4">
                          <div className="flex items-start gap-4">
                            {onBulkAssign && (
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
                            )}

                            <div className="flex-1 min-w-0">
                              <div className="flex items-start justify-between mb-2">
                                <div>
                                  <h3 className="font-medium truncate">{template.name}</h3>
                                  <p className="text-sm text-gray-600 mt-1">{template.description}</p>
                                </div>
                                
                                <div className="flex items-center gap-2 ml-4">
                                  <div className={`flex items-center gap-1 text-xs ${compatibility.color}`}>
                                    {compatibility.icon}
                                    {compatibility.message}
                                  </div>
                                </div>
                              </div>

                              <div className="flex items-center gap-4 text-sm text-gray-500 mb-3">
                                <div className="flex items-center gap-1">
                                  <Shield className="h-4 w-4" />
                                  {template.permissions.length} permissions
                                </div>
                                <div className="flex items-center gap-1">
                                  <Users className="h-4 w-4" />
                                  {template.assignedUserCount} users
                                </div>
                                <div>v{template.version}</div>
                                {template.scope && (
                                  <span className="px-2 py-0.5 bg-gray-100 text-gray-700 text-xs rounded">
                                    {template.scope}
                                  </span>
                                )}
                              </div>

                              {/* Assignment Form */}
                              {showingForm && (
                                <div className="mt-4 p-4 bg-gray-50 rounded-md space-y-3">
                                  <h4 className="font-medium text-sm">Assignment Options</h4>
                                  <div className="grid grid-cols-2 gap-3">
                                    <div>
                                      <label className="block text-xs font-medium text-gray-700 mb-1">
                                        Expires At (Optional)
                                      </label>
                                      <input
                                        type="datetime-local"
                                        className="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-blue-500"
                                        value={assignmentOptions.expiresAt}
                                        onChange={(e) =>
                                          setAssignmentOptions({
                                            ...assignmentOptions,
                                            expiresAt: e.target.value,
                                          })
                                        }
                                      />
                                    </div>
                                    <div>
                                      <label className="block text-xs font-medium text-gray-700 mb-1">
                                        Notes (Optional)
                                      </label>
                                      <input
                                        type="text"
                                        placeholder="Assignment reason..."
                                        className="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-blue-500"
                                        value={assignmentOptions.notes}
                                        onChange={(e) =>
                                          setAssignmentOptions({
                                            ...assignmentOptions,
                                            notes: e.target.value,
                                          })
                                        }
                                      />
                                    </div>
                                  </div>
                                  <div className="flex gap-2">
                                    <Button
                                      size="sm"
                                      onClick={() => handleAssignTemplate(template.id)}
                                      disabled={loading}
                                      className="flex items-center gap-1"
                                    >
                                      <Plus className="h-3 w-3" />
                                      {loading ? 'Assigning...' : 'Assign Template'}
                                    </Button>
                                    <Button
                                      size="sm"
                                      variant="outline"
                                      onClick={() => setShowAssignmentForm(false)}
                                    >
                                      Cancel
                                    </Button>
                                  </div>
                                </div>
                              )}

                              {/* Action Buttons */}
                              {!showingForm && (
                                <div className="flex gap-2">
                                  <Button
                                    size="sm"
                                    onClick={() => handleAssignTemplate(template.id)}
                                    disabled={compatibility.status !== 'compatible'}
                                    className="flex items-center gap-1"
                                  >
                                    <Plus className="h-3 w-3" />
                                    Assign
                                  </Button>
                                  <Button
                                    size="sm"
                                    variant="outline"
                                    className="flex items-center gap-1"
                                  >
                                    <Eye className="h-3 w-3" />
                                    Preview
                                  </Button>
                                </div>
                              )}

                              {/* Tags */}
                              {template.tags.length > 0 && (
                                <div className="flex flex-wrap gap-1 mt-3">
                                  {template.tags.slice(0, 3).map(tag => (
                                    <span
                                      key={tag}
                                      className="px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full"
                                    >
                                      #{tag}
                                    </span>
                                  ))}
                                  {template.tags.length > 3 && (
                                    <span className="px-2 py-0.5 bg-gray-100 text-gray-600 text-xs rounded-full">
                                      +{template.tags.length - 3} more
                                    </span>
                                  )}
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
            </div>
          )}

          {/* Assigned Templates Tab */}
          {activeTab === 'assigned' && (
            <div className="space-y-4">
              <div className="max-h-96 overflow-y-auto space-y-3">
                {assignedTemplatesWithDetails.length === 0 ? (
                  <div className="text-center py-8">
                    <FileText className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">No templates assigned</h3>
                    <p className="text-gray-600">
                      This user has no dynamic templates assigned yet
                    </p>
                  </div>
                ) : (
                  assignedTemplatesWithDetails.map(({ assignment, template }) => {
                    if (!template) return null;

                    const isExpired = assignment.expiresAt && new Date(assignment.expiresAt) < new Date();
                    const isExpiringSoon = assignment.expiresAt && 
                      new Date(assignment.expiresAt) < new Date(Date.now() + 7 * 24 * 60 * 60 * 1000); // 7 days

                    return (
                      <Card key={assignment.id} className={isExpired ? 'bg-red-50 border-red-200' : ''}>
                        <CardContent className="p-4">
                          <div className="flex items-start justify-between">
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2 mb-2">
                                <h3 className="font-medium truncate">{template.name}</h3>
                                {isExpired && (
                                  <span className="px-2 py-0.5 bg-red-100 text-red-800 text-xs rounded">
                                    Expired
                                  </span>
                                )}
                                {isExpiringSoon && !isExpired && (
                                  <span className="px-2 py-0.5 bg-yellow-100 text-yellow-800 text-xs rounded flex items-center gap-1">
                                    <Clock className="h-3 w-3" />
                                    Expiring Soon
                                  </span>
                                )}
                              </div>

                              <p className="text-sm text-gray-600 mb-3">{template.description}</p>

                              <div className="flex items-center gap-4 text-xs text-gray-500 mb-3">
                                <div>Assigned: {new Date(assignment.assignedAt).toLocaleDateString()}</div>
                                {assignment.expiresAt && (
                                  <div>Expires: {new Date(assignment.expiresAt).toLocaleDateString()}</div>
                                )}
                                <div>{template.permissions.length} permissions</div>
                                {assignment.notes && (
                                  <div className="flex items-center gap-1">
                                    <Info className="h-3 w-3" />
                                    {assignment.notes}
                                  </div>
                                )}
                              </div>
                            </div>

                            <div className="flex items-center gap-2 ml-4">
                              <Button
                                size="sm"
                                variant="outline"
                                className="flex items-center gap-1"
                              >
                                <Eye className="h-3 w-3" />
                                View
                              </Button>
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => handleUnassignTemplate(template.id)}
                                disabled={loading}
                                className="flex items-center gap-1 text-red-600 border-red-300 hover:bg-red-50"
                              >
                                <Trash2 className="h-3 w-3" />
                                Remove
                              </Button>
                            </div>
                          </div>
                        </CardContent>
                      </Card>
                    );
                  })
                )}
              </div>
            </div>
          )}

          {/* Footer */}
          <div className="flex justify-end gap-3 mt-6 pt-4 border-t">
            <Button variant="outline" onClick={onClose}>
              Close
            </Button>
          </div>
        </div>
      </Card>
    </div>
  );
};