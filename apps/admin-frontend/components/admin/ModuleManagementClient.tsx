'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { FormField, Input, Select, Textarea } from '@/components/ui/form-components';
import { ConfirmDialog as _ConfirmDialog } from '@/components/ui/ConfirmDialog';
import { AdminService } from '@/services/adminService';
// import { useModuleAuth, ModuleAccessStatus } from '@/auth/module-ctx';
import { useAdminAuth } from '@/context/simple-admin-auth';
import { toast } from 'react-hot-toast';
import { Eye, Settings, UserPlus, Shield, AlertTriangle, Plus, Search, Filter, Lock } from 'lucide-react';

// Types matching the backend module system
interface Module {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  category: string;
  icon?: string;
  status: 'active' | 'inactive' | 'maintenance' | 'deprecated';
  version: string;
  dependencies_count: number;
  created_at: string;
  access_levels: Record<string, any>;
  default_quotas: Record<string, any>;
}

interface UserModuleAssignment {
  assignment_id: string;
  module_id: string;
  module_name: string;
  display_name: string;
  access_level: 'bronze' | 'silver' | 'gold' | 'platinum' | 'enterprise';
  status: string;
  expires_at?: string;
  assigned_at: string;
  quotas: {
    api_calls?: number;
    rate_limit_per_minute: number;
    daily_limit?: number;
    monthly_limit?: number;
    custom_limits: Record<string, number>;
  };
  restrictions: {
    ip_restrictions: string[];
    time_restrictions?: string;
    feature_restrictions: Record<string, boolean>;
    endpoint_restrictions: string[];
  };
}

interface User {
  id: string;
  email: string;
  full_name?: string;
  role: string;
  status: string;
  created_at: string;
}

interface ModuleAssignmentRequest {
  module_id: string;
  access_level: string;
  custom_quotas?: Record<string, any>;
  restrictions?: Record<string, any>;
  expires_at?: string;
}

interface AssignModulesRequest {
  user_id: string;
  assignments: ModuleAssignmentRequest[];
  reason: string;
}

const ACCESS_LEVELS = [
  { value: 'bronze', label: 'Bronze - Basic access', color: 'text-amber-600' },
  { value: 'silver', label: 'Silver - Enhanced features', color: 'text-gray-500' },
  { value: 'gold', label: 'Gold - Advanced tools', color: 'text-yellow-500' },
  { value: 'platinum', label: 'Platinum - Premium access', color: 'text-purple-600' },
  { value: 'enterprise', label: 'Enterprise - Full access', color: 'text-blue-600' },
];

const MODULE_CATEGORIES = [
  'analytics',
  'trading',
  'portfolio',
  'market-data',
  'risk-management',
  'reporting'
];

export const ModuleManagementClient: React.FC = () => {
  const { hasModuleAccess, canPerformAction, getAccessLevel: _getAccessLevel, moduleAccess: _moduleAccess } = useModuleAuth();
  const { user } = useAdminAuth();
  const [activeTab, setActiveTab] = useState<'modules' | 'assignments' | 'users'>('modules');
  const [modules, setModules] = useState<Module[]>([]);
  const [users, setUsers] = useState<User[]>([]);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [userAssignments, setUserAssignments] = useState<UserModuleAssignment[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [categoryFilter, setCategoryFilter] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [showAssignDialog, setShowAssignDialog] = useState(false);
  const [showModuleDetails, setShowModuleDetails] = useState<Module | null>(null);
  const [assignmentForm, setAssignmentForm] = useState<AssignModulesRequest>({
    user_id: '',
    assignments: [],
    reason: ''
  });

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      const [modulesRes, usersRes] = await Promise.all([
        AdminService.getModules({ category: categoryFilter, status: statusFilter, search: searchTerm }),
        AdminService.getUsers({ limit: 1000 })
      ]);
      
      if (modulesRes.success) {
        setModules(modulesRes.data.modules || []);
      }
      
      if (usersRes.success) {
        setUsers(usersRes.data.users || []);
      }
    } catch (error) {
      console.error('Failed to load data:', error);
      toast.error('Failed to load module data');
    } finally {
      setLoading(false);
    }
  }, [categoryFilter, statusFilter, searchTerm]);

  // Load initial data
  useEffect(() => {
    loadData();
  }, [loadData]);

  // Load user assignments when user is selected
  const loadUserAssignments = async (userId: string) => {
    try {
      const response = await AdminService.getUserModuleAssignments(userId);
      if (response.success) {
        setUserAssignments(response.data.assignments || []);
      } else {
        toast.error('Failed to load user assignments');
      }
    } catch (error) {
      console.error('Failed to load user assignments:', error);
      toast.error('Failed to load user assignments');
    }
  };

  // Handle user selection
  const handleUserSelect = (user: User) => {
    setSelectedUser(user);
    loadUserAssignments(user.id);
    setActiveTab('assignments');
  };

  // Handle module assignment
  const handleAssignModules = async () => {
    if (!assignmentForm.user_id || assignmentForm.assignments.length === 0) {
      toast.error('Please select a user and at least one module');
      return;
    }

    try {
      const response = await AdminService.assignModulesToUser(assignmentForm);
      if (response.success) {
        toast.success(`Successfully assigned ${response.data.successful_count} modules`);
        if (response.data.failed_count > 0) {
          toast.error(`Failed to assign ${response.data.failed_count} modules`);
        }
        setShowAssignDialog(false);
        if (selectedUser) {
          loadUserAssignments(selectedUser.id);
        }
      } else {
        toast.error('Failed to assign modules');
      }
    } catch (error) {
      console.error('Failed to assign modules:', error);
      toast.error('Failed to assign modules');
    }
  };

  // Handle module revocation
  const handleRevokeModule = async (userId: string, moduleId: string, reason: string) => {
    try {
      const response = await AdminService.revokeModuleAccess(userId, moduleId, reason);
      if (response.success) {
        toast.success('Module access revoked successfully');
        loadUserAssignments(userId);
      } else {
        toast.error('Failed to revoke module access');
      }
    } catch (error) {
      console.error('Failed to revoke module access:', error);
      toast.error('Failed to revoke module access');
    }
  };

  // Add module to assignment form
  const addModuleToAssignment = (moduleId: string) => {
    const existingIndex = assignmentForm.assignments.findIndex(a => a.module_id === moduleId);
    if (existingIndex >= 0) {
      toast.error('Module already added to assignment');
      return;
    }

    const newAssignment: ModuleAssignmentRequest = {
      module_id: moduleId,
      access_level: 'bronze',
      custom_quotas: {},
      restrictions: {},
    };

    setAssignmentForm(prev => ({
      ...prev,
      assignments: [...prev.assignments, newAssignment]
    }));
  };

  // Remove module from assignment form
  const removeModuleFromAssignment = (moduleId: string) => {
    setAssignmentForm(prev => ({
      ...prev,
      assignments: prev.assignments.filter(a => a.module_id !== moduleId)
    }));
  };

  // Update assignment in form
  const updateAssignment = (moduleId: string, updates: Partial<ModuleAssignmentRequest>) => {
    setAssignmentForm(prev => ({
      ...prev,
      assignments: prev.assignments.map(a => 
        a.module_id === moduleId ? { ...a, ...updates } : a
      )
    }));
  };

  // Filter modules based on search and filters
  const filteredModules = modules.filter(module => {
    const matchesSearch = !searchTerm || 
      module.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      module.display_name.toLowerCase().includes(searchTerm.toLowerCase());
    
    const matchesCategory = categoryFilter === 'all' || module.category === categoryFilter;
    const matchesStatus = statusFilter === 'all' || module.status === statusFilter;
    
    return matchesSearch && matchesCategory && matchesStatus;
  });

  // Get status badge color
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200';
      case 'inactive': return 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300';
      case 'maintenance': return 'bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-200';
      case 'deprecated': return 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200';
      default: return 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300';
    }
  };

  // Get access level color
  const getAccessLevelColor = (level: string) => {
    const accessLevel = ACCESS_LEVELS.find(al => al.value === level);
    return accessLevel?.color || 'text-gray-600';
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2 text-gray-600 dark:text-gray-300">Loading modules...</span>
      </div>
    );
  }

  // Check if user has admin permissions for module management - allow super_admin bypass
  const isSuperAdmin = user?.roles?.includes('super_admin') || user?.claims?.role === 'super_admin';
  const canManageModules = isSuperAdmin || (hasModuleAccess('admin') && canPerformAction('admin', 'manage_modules'));
  const canAssignModules = isSuperAdmin || (hasModuleAccess('admin') && canPerformAction('admin', 'assign_modules'));
  const canViewUsers = isSuperAdmin || (hasModuleAccess('admin') && canPerformAction('admin', 'view_users'));

  // Show access denied if user doesn't have permission
  if (!canManageModules) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-center max-w-md">
          <Lock className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">Access Denied</h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            You don&apos;t have permission to manage modules. This feature requires admin-level access.
          </p>
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-700 rounded-lg p-4">
            <h3 className="font-medium text-blue-900 dark:text-blue-100 mb-2">Your Current Access:</h3>
            <ModuleAccessStatus />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">Module Management</h1>
            <p className="text-gray-600 dark:text-gray-300">Manage modules and user access assignments</p>
          </div>
          <div className="text-right">
            <div className="text-sm text-gray-500 dark:text-gray-400 mb-1">Your Access Level</div>
            <ModuleAccessStatus />
          </div>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-gray-100 dark:bg-gray-800 p-1 rounded-lg w-fit">
        <button
          onClick={() => setActiveTab('modules')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            activeTab === 'modules'
              ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
              : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
          }`}
        >
          <Settings className="w-4 h-4 inline mr-2" />
          Modules
        </button>
        <button
          onClick={() => canAssignModules && setActiveTab('assignments')}
          disabled={!canAssignModules}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            activeTab === 'assignments'
              ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
              : canAssignModules 
                ? 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
                : 'text-gray-400 dark:text-gray-500 cursor-not-allowed'
          }`}
          title={!canAssignModules ? 'Insufficient permissions to manage assignments' : undefined}
        >
          <Shield className="w-4 h-4 inline mr-2" />
          Assignments
          {!canAssignModules && <Lock className="w-3 h-3 inline ml-1" />}
        </button>
        <button
          onClick={() => canViewUsers && setActiveTab('users')}
          disabled={!canViewUsers}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            activeTab === 'users'
              ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
              : canViewUsers 
                ? 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
                : 'text-gray-400 dark:text-gray-500 cursor-not-allowed'
          }`}
          title={!canViewUsers ? 'Insufficient permissions to view users' : undefined}
        >
          <UserPlus className="w-4 h-4 inline mr-2" />
          Users
          {!canViewUsers && <Lock className="w-3 h-3 inline ml-1" />}
        </button>
      </div>

      {/* Modules Tab */}
      {activeTab === 'modules' && (
        <div className="space-y-6">
          {/* Search and Filters */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                <Input
                  type="text"
                  placeholder="Search modules..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
              <Select
                value={categoryFilter}
                onChange={(e) => setCategoryFilter(e.target.value)}
              >
                <option value="all">All Categories</option>
                {MODULE_CATEGORIES.map(cat => (
                  <option key={cat} value={cat}>
                    {cat.charAt(0).toUpperCase() + cat.slice(1).replace('-', ' ')}
                  </option>
                ))}
              </Select>
              <Select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
              >
                <option value="all">All Status</option>
                <option value="active">Active</option>
                <option value="inactive">Inactive</option>
                <option value="maintenance">Maintenance</option>
                <option value="deprecated">Deprecated</option>
              </Select>
              <Button onClick={loadData} variant="outline">
                <Filter className="w-4 h-4 mr-2" />
                Apply Filters
              </Button>
            </div>
          </div>

          {/* Modules Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredModules.map((module) => (
              <div key={module.id} className="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-md transition-shadow p-6">
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center space-x-3">
                    {module.icon && (
                      <div className="w-10 h-10 bg-blue-100 dark:bg-blue-900/50 rounded-lg flex items-center justify-center">
                        <span className="text-xl">{module.icon}</span>
                      </div>
                    )}
                    <div>
                      <h3 className="font-semibold text-gray-900 dark:text-gray-100">{module.display_name}</h3>
                      <p className="text-sm text-gray-500 dark:text-gray-400">{module.name}</p>
                    </div>
                  </div>
                  <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(module.status)}`}>
                    {module.status}
                  </span>
                </div>
                
                <p className="text-gray-600 dark:text-gray-300 text-sm mb-4 line-clamp-2">
                  {module.description || 'No description available'}
                </p>
                
                <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 mb-4">
                  <span>Category: {module.category}</span>
                  <span>v{module.version}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-500 dark:text-gray-400">
                    {module.dependencies_count} dependencies
                  </span>
                  <div className="flex space-x-2">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => setShowModuleDetails(module)}
                    >
                      <Eye className="w-4 h-4" />
                    </Button>
                    <Button
                      size="sm"
                      onClick={() => addModuleToAssignment(module.id)}
                    >
                      <Plus className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Assignments Tab */}
      {activeTab === 'assignments' && (
        <div className="space-y-6">
          {selectedUser ? (
            <div>
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 mb-6">
                <div className="flex items-center justify-between mb-4">
                  <div>
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                      {selectedUser.full_name || selectedUser.email}
                    </h2>
                    <p className="text-gray-500 dark:text-gray-400">{selectedUser.email}</p>
                  </div>
                  <div className="flex space-x-2">
                    <Button
                      onClick={() => {
                        setAssignmentForm(prev => ({ ...prev, user_id: selectedUser.id }));
                        setShowAssignDialog(true);
                      }}
                    >
                      <Plus className="w-4 h-4 mr-2" />
                      Assign Modules
                    </Button>
                    <Button variant="outline" onClick={() => setSelectedUser(null)}>
                      Back to Users
                    </Button>
                  </div>
                </div>
              </div>

              {/* User Assignments */}
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
                <div className="p-6 border-b">
                  <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">Module Assignments</h3>
                </div>
                <div className="divide-y">
                  {userAssignments.length === 0 ? (
                    <div className="p-6 text-center text-gray-500 dark:text-gray-400">
                      No modules assigned to this user
                    </div>
                  ) : (
                    userAssignments.map((assignment) => (
                      <div key={assignment.assignment_id} className="p-6">
                        <div className="flex items-center justify-between mb-2">
                          <div>
                            <h4 className="font-medium text-gray-900 dark:text-gray-100">{assignment.display_name}</h4>
                            <p className="text-sm text-gray-500 dark:text-gray-400">{assignment.module_name}</p>
                          </div>
                          <div className="flex items-center space-x-2">
                            <span className={`text-sm font-medium ${getAccessLevelColor(assignment.access_level)}`}>
                              {assignment.access_level.toUpperCase()}
                            </span>
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => {
                                const reason = prompt('Reason for revoking access:');
                                if (reason) {
                                  handleRevokeModule(selectedUser.id, assignment.module_id, reason);
                                }
                              }}
                            >
                              <AlertTriangle className="w-4 h-4" />
                            </Button>
                          </div>
                        </div>
                        
                        <div className="grid grid-cols-2 gap-4 text-sm text-gray-600 dark:text-gray-300">
                          <div>
                            <span className="font-medium">Assigned:</span> {new Date(assignment.assigned_at).toLocaleDateString()}
                          </div>
                          {assignment.expires_at && (
                            <div>
                              <span className="font-medium">Expires:</span> {new Date(assignment.expires_at).toLocaleDateString()}
                            </div>
                          )}
                          <div>
                            <span className="font-medium">Rate Limit:</span> {assignment.quotas.rate_limit_per_minute}/min
                          </div>
                          <div>
                            <span className="font-medium">Status:</span> {assignment.status}
                          </div>
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>
          ) : (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
              <UserPlus className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">Select a User</h3>
              <p className="text-gray-500 dark:text-gray-400 mb-4">Choose a user from the Users tab to view and manage their module assignments</p>
              <Button onClick={() => setActiveTab('users')}>Go to Users</Button>
            </div>
          )}
        </div>
      )}

      {/* Users Tab */}
      {activeTab === 'users' && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
          <div className="p-6 border-b">
            <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">Users</h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">Select a user to manage their module assignments</p>
          </div>
          <div className="divide-y">
            {users.map((user) => (
              <div key={user.id} className="p-4 hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
                   onClick={() => handleUserSelect(user)}>
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-gray-900 dark:text-gray-100">{user.full_name || user.email}</h4>
                    <p className="text-sm text-gray-500 dark:text-gray-400">{user.email}</p>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className="px-2 py-1 bg-blue-100 dark:bg-blue-900/50 text-blue-800 dark:text-blue-200 rounded-full text-xs font-medium">
                      {user.role}
                    </span>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                      user.status === 'active' 
                        ? 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200' 
                        : 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200'
                    }`}>
                      {user.status}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Module Assignment Dialog */}
      {showAssignDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-60">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] overflow-hidden">
            <div className="p-6 border-b">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Assign Modules</h2>
              <p className="text-sm text-gray-500 dark:text-gray-400">Select modules and configure access levels</p>
            </div>
            
            <div className="p-6 max-h-[70vh] overflow-y-auto">
              <div className="mb-6">
                <FormField label="User">
                  <Select
                    value={assignmentForm.user_id}
                    onChange={(e) => setAssignmentForm(prev => ({ ...prev, user_id: e.target.value }))}
                    required
                  >
                    <option value="">Select a user...</option>
                    {users.map(user => (
                      <option key={user.id} value={user.id}>
                        {user.full_name || user.email} ({user.email})
                      </option>
                    ))}
                  </Select>
                </FormField>
              </div>

              <div className="mb-6">
                <FormField label="Reason">
                  <Textarea
                    value={assignmentForm.reason}
                    onChange={(e) => setAssignmentForm(prev => ({ ...prev, reason: e.target.value }))}
                    placeholder="Reason for module assignment..."
                    required
                  />
                </FormField>
              </div>

              {/* Selected Modules */}
              {assignmentForm.assignments.length > 0 && (
                <div className="mb-6">
                  <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-3">Selected Modules</h3>
                  <div className="space-y-4">
                    {assignmentForm.assignments.map((assignment) => {
                      const module = modules.find(m => m.id === assignment.module_id);
                      return (
                        <div key={assignment.module_id} className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
                          <div className="flex items-center justify-between mb-3">
                            <div>
                              <h4 className="font-medium text-gray-900 dark:text-gray-100">{module?.display_name}</h4>
                              <p className="text-sm text-gray-500 dark:text-gray-400">{module?.name}</p>
                            </div>
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => removeModuleFromAssignment(assignment.module_id)}
                            >
                              Remove
                            </Button>
                          </div>
                          
                          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <FormField label="Access Level">
                              <Select
                                value={assignment.access_level}
                                onChange={(e) => updateAssignment(assignment.module_id, { access_level: e.target.value })}
                              >
                                {ACCESS_LEVELS.map(level => (
                                  <option key={level.value} value={level.value}>
                                    {level.label}
                                  </option>
                                ))}
                              </Select>
                            </FormField>
                            
                            <FormField label="Expires At (Optional)">
                              <Input
                                type="datetime-local"
                                value={assignment.expires_at || ''}
                                onChange={(e) => updateAssignment(assignment.module_id, { expires_at: e.target.value || undefined })}
                              />
                            </FormField>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Available Modules */}
              <div>
                <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-3">Available Modules</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-60 overflow-y-auto">
                  {modules
                    .filter(module => !assignmentForm.assignments.some(a => a.module_id === module.id))
                    .map((module) => (
                      <div key={module.id} className="p-3 border border-gray-200 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
                           onClick={() => addModuleToAssignment(module.id)}>
                        <div className="flex items-center justify-between">
                          <div>
                            <h4 className="font-medium text-gray-900 dark:text-gray-100 text-sm">{module.display_name}</h4>
                            <p className="text-xs text-gray-500 dark:text-gray-400">{module.category}</p>
                          </div>
                          <Plus className="w-4 h-4 text-blue-600" />
                        </div>
                      </div>
                    ))}
                </div>
              </div>
            </div>
            
            <div className="p-6 border-t border-gray-200 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 flex justify-end space-x-3">
              <Button variant="outline" onClick={() => setShowAssignDialog(false)}>
                Cancel
              </Button>
              <Button onClick={handleAssignModules}>
                Assign Modules
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Module Details Dialog */}
      {showModuleDetails && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-60">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-hidden">
            <div className="p-6 border-b">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{showModuleDetails.display_name}</h2>
              <p className="text-sm text-gray-500 dark:text-gray-400">{showModuleDetails.name}</p>
            </div>
            
            <div className="p-6 max-h-[70vh] overflow-y-auto">
              <div className="space-y-4">
                <div>
                  <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Description</h3>
                  <p className="text-gray-600 dark:text-gray-300">{showModuleDetails.description || 'No description available'}</p>
                </div>
                
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Category</h3>
                    <p className="text-gray-600 dark:text-gray-300">{showModuleDetails.category}</p>
                  </div>
                  <div>
                    <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Version</h3>
                    <p className="text-gray-600 dark:text-gray-300">v{showModuleDetails.version}</p>
                  </div>
                  <div>
                    <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Status</h3>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(showModuleDetails.status)}`}>
                      {showModuleDetails.status}
                    </span>
                  </div>
                  <div>
                    <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Dependencies</h3>
                    <p className="text-gray-600 dark:text-gray-300">{showModuleDetails.dependencies_count} modules</p>
                  </div>
                </div>
                
                <div>
                  <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Access Levels</h3>
                  <div className="space-y-2">
                    {Object.keys(showModuleDetails.access_levels || {}).map(level => (
                      <div key={level} className={`text-sm ${getAccessLevelColor(level)}`}>
                        {level.toUpperCase()}
                      </div>
                    ))}
                  </div>
                </div>
                
                <div>
                  <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Default Quotas</h3>
                  <pre className="text-xs bg-gray-100 dark:bg-gray-700 dark:text-gray-300 p-3 rounded overflow-x-auto">
                    {JSON.stringify(showModuleDetails.default_quotas, null, 2)}
                  </pre>
                </div>
              </div>
            </div>
            
            <div className="p-6 border-t border-gray-200 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 flex justify-end">
              <Button onClick={() => setShowModuleDetails(null)}>
                Close
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
