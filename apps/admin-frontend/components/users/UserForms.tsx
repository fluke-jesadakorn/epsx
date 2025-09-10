/**
 * User Forms - Create/Edit/Bulk Operations
 * Consolidates: CreateUserButton, UserEditForm, CustomPermissionForm,
 * BulkOperationsInterface, DragDropBulkAssignment, UserTableWithSelection,
 * PermissionExportImport, BulkPermissionManager
 */

'use client';

import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useRouter } from 'next/navigation';
import { toast } from '@/hooks/use-toast';
import { 
  User as UserIcon, 
  UserPlus, 
  Users, 
  Save, 
  X,
  Upload,
  Download,
  FileText,
  Shield,
  Calendar,
  Mail,
  AlertCircle,
  CheckCircle,
  Clock,
  Eye,
  ArrowLeft,
  Edit3,
  UserCheck,
  Settings,
  Lock,
  Unlock
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import { Progress } from '@/components/ui/progress';

import type { User, Permission } from '@/types/core';
import type { SelectOption } from '@/types/ui';
import { adminClient } from '@/lib/api/unified-admin-client';

// Form validation schemas
const createUserSchema = z.object({
  email: z.string().email('Invalid email format'),
  firstName: z.string().min(1, 'First name is required').optional(),
  lastName: z.string().min(1, 'Last name is required').optional(),
  displayName: z.string().optional(),
  role: z.enum(['admin', 'user', 'premium_user']),
  packageTier: z.string(),
  permissions: z.array(z.string()),
  isActive: z.boolean()
});

const editUserSchema = createUserSchema.partial().extend({
  id: z.string().min(1, 'User ID is required')
});

const bulkUserSchema = z.object({
  userIds: z.array(z.string()).min(1, 'At least one user is required'),
  operation: z.enum(['update_role', 'update_tier', 'assign_permissions', 'activate', 'deactivate', 'delete']),
  role: z.string().optional(),
  packageTier: z.string().optional(),
  permissions: z.array(z.string()).optional(),
  reason: z.string().min(3, 'Reason is required for bulk operations')
});

type CreateUserForm = z.infer<typeof createUserSchema>;
type EditUserForm = z.infer<typeof editUserSchema>;
type BulkUserForm = z.infer<typeof bulkUserSchema>;

interface UserFormsProps {
  mode?: 'create' | 'edit' | 'bulk' | 'view';
  currentUser?: any;
  users?: User[];
  editUser?: User;
  availableRoles?: SelectOption[];
  availablePackageTiers?: SelectOption[];
  availablePermissions?: Array<{
    id: string;
    name: string;
    description: string;
    category: string;
  }>;
  onUserCreated?: (user: User) => void;
  onUserUpdated?: (user: User) => void;
  onBulkOperationComplete?: (result: any) => void;
  className?: string;
}

export function UserForms({
  mode = 'create',
  currentUser,
  users = [],
  editUser,
  availableRoles = [],
  availablePackageTiers = [],
  availablePermissions = [],
  onUserCreated,
  onUserUpdated,
  onBulkOperationComplete,
  className = ''
}: UserFormsProps) {
  const router = useRouter();
  // State management
  const [activeTab, setActiveTab] = useState(mode);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [bulkOperation, setBulkOperation] = useState<string>('');
  const [importProgress, setImportProgress] = useState(0);
  const [isImporting, setIsImporting] = useState(false);
  const [formStep, setFormStep] = useState(1);
  const [previewMode, setPreviewMode] = useState(false);
  
  // Create User Form
  const createForm = useForm<CreateUserForm>({
    resolver: zodResolver(createUserSchema),
    defaultValues: {
      email: '',
      firstName: '',
      lastName: '',
      displayName: '',
      role: 'user',
      packageTier: 'basic',
      permissions: [],
      isActive: true
    }
  });

  // Edit User Form
  const editForm = useForm<EditUserForm>({
    resolver: zodResolver(editUserSchema)
  });

  // Bulk Operations Form
  const bulkForm = useForm<BulkUserForm>({
    resolver: zodResolver(bulkUserSchema),
    defaultValues: {
      userIds: [],
      operation: 'update_role',
      reason: ''
    }
  });

  // Default options if not provided
  const defaultRoles: SelectOption[] = availableRoles.length > 0 ? availableRoles : [
    { label: 'Admin', value: 'admin' },
    { label: 'User', value: 'user' },
    { label: 'Premium User', value: 'premium_user' }
  ];

  const defaultTiers: SelectOption[] = availablePackageTiers.length > 0 ? availablePackageTiers : [
    { label: 'Basic', value: 'basic' },
    { label: 'Premium', value: 'premium' },
    { label: 'Pro', value: 'pro' },
    { label: 'Enterprise', value: 'enterprise' }
  ];

  // Create user handler
  const handleCreateUser = async (data: CreateUserForm) => {
    if (previewMode) {
      setPreviewMode(false);
      return;
    }
    
    setIsSubmitting(true);
    try {
      const response = await adminClient.createUser({
        email: data.email,
        permissions: data.permissions,
        display_name: data.displayName,
        first_name: data.firstName,
        last_name: data.lastName
      });

      if (response.success && response.data) {
        const newUser: User = {
          id: response.data.userId,
          email: data.email,
          name: data.displayName || `${data.firstName || ''} ${data.lastName || ''}`.trim(),
          firstName: data.firstName,
          lastName: data.lastName,
          displayName: data.displayName,
          role: data.role,
          packageTier: data.packageTier,
          permissions: data.permissions,
          isActive: data.isActive,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString()
        };

        toast({
          title: "User created successfully",
          description: `${data.email} has been added to the system.`,
          variant: "default",
        });

        onUserCreated?.(newUser);
        createForm.reset();
        
        // Redirect to users list after successful creation
        setTimeout(() => {
          router.push('/users');
        }, 1500);
      } else {
        toast({
          title: "Error creating user",
          description: response.error || "Failed to create user. Please try again.",
          variant: "destructive",
        });
      }
    } catch (error) {
      console.error('Create user error:', error);
      toast({
        title: "Error",
        description: "Network error. Please check your connection and try again.",
        variant: "destructive",
      });
    } finally {
      setIsSubmitting(false);
    }
  };
  
  const handlePreviewUser = () => {
    setPreviewMode(true);
  };
  
  const handleNextStep = () => {
    if (formStep < 3) {
      setFormStep(formStep + 1);
    }
  };
  
  const handlePrevStep = () => {
    if (formStep > 1) {
      setFormStep(formStep - 1);
    }
  };

  // Edit user handler
  const handleEditUser = async (data: EditUserForm) => {
    if (previewMode) {
      setPreviewMode(false);
      return;
    }
    
    setIsSubmitting(true);
    try {
      const updateData = {
        email: data.email,
        display_name: data.displayName,
        first_name: data.firstName,
        last_name: data.lastName,
        role: data.role,
        package_tier: data.packageTier,
        is_active: data.isActive,
        permissions: data.permissions || []
      };
      
      const response = await adminClient.updateUser(data.id!, updateData);

      if (response.success) {
        const updatedUser: User = {
          id: data.id!,
          email: data.email || editUser?.email || '',
          name: data.displayName || `${data.firstName || ''} ${data.lastName || ''}`.trim(),
          firstName: data.firstName,
          lastName: data.lastName,
          displayName: data.displayName,
          role: data.role || editUser?.role || 'user',
          packageTier: data.packageTier || editUser?.packageTier || 'basic',
          permissions: data.permissions || editUser?.permissions || [],
          isActive: data.isActive !== undefined ? data.isActive : editUser?.isActive || true,
          createdAt: editUser?.createdAt || new Date().toISOString(),
          updatedAt: new Date().toISOString()
        };
        
        toast({
          title: "User updated successfully",
          description: `${data.email || editUser?.email} has been updated.`,
          variant: "default",
        });
        
        onUserUpdated?.(updatedUser);
        editForm.reset();
        
        // Redirect to user profile after successful edit
        setTimeout(() => {
          router.push(`/users/${data.id}`);
        }, 1500);
      } else {
        toast({
          title: "Error updating user",
          description: response.error || "Failed to update user. Please try again.",
          variant: "destructive",
        });
      }
    } catch (error) {
      console.error('Edit user error:', error);
      toast({
        title: "Error",
        description: "Network error. Please check your connection and try again.",
        variant: "destructive",
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  // Bulk operations handler
  const handleBulkOperation = async (data: BulkUserForm) => {
    setIsSubmitting(true);
    try {
      // Mock bulk operation - replace with actual API call
      const result = {
        operation: data.operation,
        userIds: data.userIds,
        successful: data.userIds.length,
        failed: 0,
        reason: data.reason
      };

      onBulkOperationComplete?.(result);
      bulkForm.reset();
      setSelectedUsers([]);
    } catch (error) {
      console.error('Bulk operation error:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  // CSV Import handler
  const handleCSVImport = async (file: File) => {
    setIsImporting(true);
    setImportProgress(0);

    try {
      // Mock CSV import progress
      for (let i = 0; i <= 100; i += 10) {
        setImportProgress(i);
        await new Promise(resolve => setTimeout(resolve, 200));
      }

      // In a real implementation, parse CSV and create users
      console.log('CSV import completed:', file.name);
    } catch (error) {
      console.error('CSV import error:', error);
    } finally {
      setIsImporting(false);
      setImportProgress(0);
    }
  };

  // Load user for editing
  const loadUserForEdit = (user: User) => {
    editForm.reset({
      id: user.id,
      email: user.email,
      firstName: user.firstName,
      lastName: user.lastName,
      displayName: user.displayName,
      role: user.role,
      packageTier: user.packageTier,
      permissions: user.permissions || [],
      isActive: user.isActive
    });
    setActiveTab('edit');
  };
  
  // Initialize edit form with existing user data
  useEffect(() => {
    if (mode === 'edit' && editUser) {
      editForm.reset({
        id: editUser.id,
        email: editUser.email,
        firstName: editUser.firstName,
        lastName: editUser.lastName,
        displayName: editUser.displayName,
        role: editUser.role,
        packageTier: editUser.packageTier,
        permissions: editUser.permissions || [],
        isActive: editUser.isActive
      });
    }
  }, [editUser, mode, editForm]);

  // Show single form based on mode
  if (mode === 'edit') {
    return (
      <div className={`space-y-6 ${className}`}>
        {/* Background Decorations */}
        <div className="fixed inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-cyan-500/20 rounded-full blur-xl animate-pulse"></div>
          <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
          <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-indigo-400/15 to-blue-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
        </div>
        
        <div className="relative">
          {/* Header */}
          <div className="text-center mb-8">
            <div className="relative inline-block">
              <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-blue-600 via-purple-600 via-indigo-600 to-cyan-600 bg-clip-text text-transparent mb-4">
                ✏️ Edit User Profile
              </h1>
              <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full animate-pulse"></div>
            </div>
            <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
              Update user information, roles, and permissions for {editUser?.email || 'the user'}
            </p>
          </div>
          
          {/* Breadcrumb Navigation */}
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-gray-100/50 to-gray-200/50 dark:from-gray-800/50 dark:to-gray-700/50 p-0.5 mb-8">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-4">
              <div className="flex items-center gap-2 text-sm">
                <button
                  onClick={() => router.push('/users')}
                  className="flex items-center gap-2 text-blue-600 hover:text-blue-700 transition-colors"
                >
                  <ArrowLeft className="w-4 h-4" />
                  Users
                </button>
                <span className="text-gray-400 dark:text-gray-500">/</span>
                <button
                  onClick={() => router.push(`/users/${editUser?.id}`)}
                  className="text-blue-600 hover:text-blue-700 transition-colors"
                >
                  {editUser?.displayName || editUser?.email || 'User Profile'}
                </button>
                <span className="text-gray-400 dark:text-gray-500">/</span>
                <span className="text-gray-600 dark:text-gray-300">Edit</span>
              </div>
            </div>
          </div>
          
          {/* Progress Steps */}
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-indigo-400/20 p-0.5 mb-8">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full blur-sm animate-pulse opacity-60"></div>
              <div className="flex items-center justify-between">
                <div className={`flex items-center gap-2 ${formStep >= 1 ? 'text-blue-600' : 'text-gray-400'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${formStep >= 1 ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-500'}`}>
                    {formStep > 1 ? <CheckCircle className="w-4 h-4" /> : '1'}
                  </div>
                  <span className="font-medium">User Information</span>
                </div>
                <div className={`h-0.5 flex-1 mx-4 ${formStep >= 2 ? 'bg-blue-500' : 'bg-gray-200'}`}></div>
                <div className={`flex items-center gap-2 ${formStep >= 2 ? 'text-blue-600' : 'text-gray-400'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${formStep >= 2 ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-500'}`}>
                    {formStep > 2 ? <CheckCircle className="w-4 h-4" /> : '2'}
                  </div>
                  <span className="font-medium">Access Control</span>
                </div>
                <div className={`h-0.5 flex-1 mx-4 ${formStep >= 3 ? 'bg-blue-500' : 'bg-gray-200'}`}></div>
                <div className={`flex items-center gap-2 ${formStep >= 3 ? 'text-blue-600' : 'text-gray-400'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${formStep >= 3 ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-500'}`}>
                    {previewMode ? <CheckCircle className="w-4 h-4" /> : '3'}
                  </div>
                  <span className="font-medium">Review & Update</span>
                </div>
              </div>
            </div>
          </div>

          {/* Edit User Form */}
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-indigo-400/20 to-purple-400/20 p-0.5">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-indigo-500 rounded-full blur-sm animate-pulse opacity-60"></div>
              
              <form onSubmit={editForm.handleSubmit(previewMode ? handleEditUser : () => setPreviewMode(true))} className="space-y-6">
                {/* Step 1: User Information */}
                {formStep === 1 && (
                  <div className="space-y-6">
                    <div className="flex items-center gap-2 mb-6">
                      <Edit3 className="w-6 h-6 text-blue-500" />
                      <h3 className="text-xl font-semibold text-gray-800 dark:text-gray-200">User Information</h3>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div className="space-y-2">
                        <Label htmlFor="email" className="text-sm font-medium">
                          Email Address <span className="text-red-500">*</span>
                        </Label>
                        <div className="relative">
                          <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                          <Input
                            {...editForm.register('email')}
                            placeholder="user@example.com"
                            className="pl-12 h-12 rounded-2xl border-2 border-blue-200 dark:border-blue-700 bg-gradient-to-r from-blue-50/50 to-indigo-50/50 dark:from-blue-900/20 dark:to-indigo-900/20"
                          />
                        </div>
                        {editForm.formState.errors.email && (
                          <p className="text-sm text-red-500 flex items-center gap-1">
                            <AlertCircle className="w-4 h-4" />
                            {editForm.formState.errors.email.message}
                          </p>
                        )}
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="displayName" className="text-sm font-medium">
                          Display Name
                        </Label>
                        <Input
                          {...editForm.register('displayName')}
                          placeholder="John Doe"
                          className="h-12 rounded-2xl border-2 border-purple-200 dark:border-purple-700 bg-gradient-to-r from-purple-50/50 to-indigo-50/50 dark:from-purple-900/20 dark:to-indigo-900/20"
                        />
                      </div>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div className="space-y-2">
                        <Label htmlFor="firstName" className="text-sm font-medium">
                          First Name
                        </Label>
                        <Input
                          {...editForm.register('firstName')}
                          placeholder="John"
                          className="h-12 rounded-2xl border-2 border-indigo-200 dark:border-indigo-700"
                        />
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="lastName" className="text-sm font-medium">
                          Last Name
                        </Label>
                        <Input
                          {...editForm.register('lastName')}
                          placeholder="Doe"
                          className="h-12 rounded-2xl border-2 border-cyan-200 dark:border-cyan-700"
                        />
                      </div>
                    </div>
                  </div>
                )}

                {/* Step 2: Access Control */}
                {formStep === 2 && (
                  <div className="space-y-6">
                    <div className="flex items-center gap-2 mb-6">
                      <Shield className="w-6 h-6 text-indigo-500" />
                      <h3 className="text-xl font-semibold text-gray-800 dark:text-gray-200">Access Control</h3>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div className="space-y-2">
                        <Label htmlFor="role" className="text-sm font-medium">
                          User Role <span className="text-red-500">*</span>
                        </Label>
                        <Select
                          value={editForm.watch('role') || ''}
                          onValueChange={(value: any) => editForm.setValue('role', value)}
                        >
                          <SelectTrigger className="h-12 rounded-2xl border-2 border-yellow-200 dark:border-yellow-700">
                            <SelectValue placeholder="Select user role" />
                          </SelectTrigger>
                          <SelectContent className="rounded-2xl">
                            {defaultRoles.map(role => (
                              <SelectItem key={role.value} value={role.value}>
                                {role.value === 'admin' && '👑 '}
                                {role.value === 'premium_user' && '⭐ '}
                                {role.value === 'user' && '👤 '}
                                {role.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                        {editForm.formState.errors.role && (
                          <p className="text-sm text-red-500 flex items-center gap-1">
                            <AlertCircle className="w-4 h-4" />
                            {editForm.formState.errors.role.message}
                          </p>
                        )}
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="packageTier" className="text-sm font-medium">
                          Package Tier <span className="text-red-500">*</span>
                        </Label>
                        <Select
                          value={editForm.watch('packageTier') || ''}
                          onValueChange={(value) => editForm.setValue('packageTier', value)}
                        >
                          <SelectTrigger className="h-12 rounded-2xl border-2 border-orange-200 dark:border-orange-700">
                            <SelectValue placeholder="Select package tier" />
                          </SelectTrigger>
                          <SelectContent className="rounded-2xl">
                            {defaultTiers.map(tier => (
                              <SelectItem key={tier.value} value={tier.value}>
                                {tier.value === 'basic' && '🥉 '}
                                {tier.value === 'premium' && '🥈 '}
                                {tier.value === 'pro' && '🥇 '}
                                {tier.value === 'enterprise' && '💎 '}
                                {tier.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    </div>

                    {/* Permissions */}
                    <div className="space-y-2">
                      <Label className="text-sm font-medium">User Permissions</Label>
                      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-gray-100/50 to-gray-200/50 dark:from-gray-800/50 dark:to-gray-700/50 p-0.5">
                        <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-sm rounded-2xl p-4 max-h-64 overflow-y-auto">
                          {availablePermissions.length > 0 ? (
                            <div className="space-y-2">
                              {availablePermissions.map(permission => (
                                <label key={permission.id} className="flex items-start gap-3 p-3 hover:bg-gray-50 dark:hover:bg-gray-800 rounded-xl cursor-pointer transition-colors">
                                  <Checkbox
                                    checked={(editForm.watch('permissions') || []).includes(permission.id)}
                                    onCheckedChange={(checked) => {
                                      const currentPermissions = editForm.watch('permissions') || [];
                                      if (checked) {
                                        editForm.setValue('permissions', [...currentPermissions, permission.id]);
                                      } else {
                                        editForm.setValue('permissions', currentPermissions.filter(p => p !== permission.id));
                                      }
                                    }}
                                  />
                                  <div className="flex-1">
                                    <div className="font-medium text-sm">{permission.name}</div>
                                    <div className="text-xs text-gray-500 dark:text-gray-400">{permission.description}</div>
                                    <Badge variant="secondary" size="sm" className="mt-1">
                                      {permission.category}
                                    </Badge>
                                  </div>
                                </label>
                              ))}
                            </div>
                          ) : (
                            <div className="text-center py-8">
                              <Shield className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                              <p className="text-gray-500">No permissions available to assign</p>
                            </div>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* Account Status */}
                    <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-2xl">
                      <div className="flex items-center gap-3">
                        {editForm.watch('isActive') ? (
                          <Unlock className="w-5 h-5 text-green-500" />
                        ) : (
                          <Lock className="w-5 h-5 text-red-500" />
                        )}
                        <div>
                          <Label className="text-sm font-medium">Account Status</Label>
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            {editForm.watch('isActive') ? 'Account is active and can access the system' : 'Account is disabled and cannot access the system'}
                          </p>
                        </div>
                      </div>
                      <Switch
                        checked={editForm.watch('isActive')}
                        onCheckedChange={(checked) => editForm.setValue('isActive', checked)}
                      />
                    </div>
                  </div>
                )}

                {/* Step 3: Review */}
                {(formStep === 3 || previewMode) && (
                  <div className="space-y-6">
                    <div className="flex items-center gap-2 mb-6">
                      <CheckCircle className="w-6 h-6 text-green-500" />
                      <h3 className="text-xl font-semibold text-gray-800 dark:text-gray-200">
                        {previewMode ? 'Confirm Changes' : 'Review Changes'}
                      </h3>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div className="space-y-4">
                        <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-2xl">
                          <h4 className="font-semibold text-blue-800 dark:text-blue-200 mb-2">Personal Information</h4>
                          <div className="space-y-1 text-sm">
                            <p><strong>Email:</strong> {editForm.watch('email') || 'Not changed'}</p>
                            <p><strong>Display Name:</strong> {editForm.watch('displayName') || 'Not provided'}</p>
                            <p><strong>First Name:</strong> {editForm.watch('firstName') || 'Not provided'}</p>
                            <p><strong>Last Name:</strong> {editForm.watch('lastName') || 'Not provided'}</p>
                          </div>
                        </div>
                        
                        <div className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded-2xl">
                          <h4 className="font-semibold text-purple-800 dark:text-purple-200 mb-2">Access Control</h4>
                          <div className="space-y-1 text-sm">
                            <p><strong>Role:</strong> {editForm.watch('role') || 'Not changed'}</p>
                            <p><strong>Package Tier:</strong> {editForm.watch('packageTier') || 'Not changed'}</p>
                            <p className="flex items-center gap-2">
                              <strong>Status:</strong>
                              {editForm.watch('isActive') ? (
                                <Badge variant="default" className="text-xs">Active</Badge>
                              ) : (
                                <Badge variant="secondary" className="text-xs">Inactive</Badge>
                              )}
                            </p>
                          </div>
                        </div>
                      </div>
                      
                      <div className="space-y-4">
                        <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-2xl">
                          <h4 className="font-semibold text-green-800 dark:text-green-200 mb-2">Permissions</h4>
                          {(editForm.watch('permissions') || []).length > 0 ? (
                            <div className="space-y-1">
                              {(editForm.watch('permissions') || []).map((permId, index) => {
                                const perm = availablePermissions.find(p => p.id === permId);
                                return (
                                  <div key={index} className="flex items-center gap-2 text-sm">
                                    <Shield className="w-3 h-3 text-green-600" />
                                    {perm?.name || permId}
                                  </div>
                                );
                              })}
                            </div>
                          ) : (
                            <p className="text-sm text-gray-500">No permissions assigned</p>
                          )}
                        </div>
                        
                        <div className="p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-2xl">
                          <h4 className="font-semibold text-yellow-800 dark:text-yellow-200 mb-2">Original User</h4>
                          <div className="space-y-1 text-sm text-gray-600 dark:text-gray-400">
                            <p><strong>Original Email:</strong> {editUser?.email}</p>
                            <p><strong>Original Role:</strong> {editUser?.role}</p>
                            <p><strong>Original Status:</strong> {editUser?.isActive ? 'Active' : 'Inactive'}</p>
                            <p><strong>Created:</strong> {editUser?.createdAt ? new Date(editUser.createdAt).toLocaleDateString() : 'Unknown'}</p>
                          </div>
                        </div>
                      </div>
                    </div>
                    
                    {previewMode && (
                      <div className="p-4 bg-orange-50 dark:bg-orange-900/20 border-2 border-orange-200 dark:border-orange-700 rounded-2xl">
                        <div className="flex items-center gap-2 mb-2">
                          <AlertCircle className="w-5 h-5 text-orange-600" />
                          <h4 className="font-semibold text-orange-800 dark:text-orange-200">Ready to Update</h4>
                        </div>
                        <p className="text-sm text-orange-700 dark:text-orange-300">
                          Please review the changes above. Once you click "Update User", the changes will be applied to the user account immediately.
                        </p>
                      </div>
                    )}
                  </div>
                )}

                {/* Navigation Buttons */}
                <div className="flex justify-between pt-6 border-t border-gray-200 dark:border-gray-700">
                  <div>
                    {formStep > 1 && !previewMode && (
                      <Button
                        type="button"
                        variant="outline"
                        onClick={handlePrevStep}
                        className="rounded-2xl"
                      >
                        Previous
                      </Button>
                    )}
                    {previewMode && (
                      <Button
                        type="button"
                        variant="outline"
                        onClick={() => setPreviewMode(false)}
                        className="rounded-2xl"
                      >
                        Back to Edit
                      </Button>
                    )}
                  </div>
                  
                  <div className="flex gap-3">
                    <Button
                      type="button"
                      variant="outline"
                      onClick={() => {
                        if (confirm('Are you sure? All changes will be lost.')) {
                          router.push(`/users/${editUser?.id}`);
                        }
                      }}
                      className="rounded-2xl"
                    >
                      Cancel
                    </Button>
                    
                    {formStep < 3 && !previewMode && (
                      <Button
                        type="button"
                        onClick={handleNextStep}
                        className="bg-gradient-to-r from-blue-500 to-indigo-500 hover:from-blue-600 hover:to-indigo-600 rounded-2xl"
                      >
                        Next Step
                      </Button>
                    )}
                    
                    {(formStep === 3 || previewMode) && (
                      <Button
                        type="submit"
                        disabled={isSubmitting}
                        className={`rounded-2xl ${
                          previewMode 
                            ? 'bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600'
                            : 'bg-gradient-to-r from-purple-500 to-indigo-500 hover:from-purple-600 hover:to-indigo-600'
                        }`}
                      >
                        {isSubmitting ? (
                          <>
                            <Clock className="w-4 h-4 mr-2 animate-spin" />
                            {previewMode ? 'Updating...' : 'Processing...'}
                          </>
                        ) : (
                          <>
                            {previewMode ? (
                              <>
                                <UserCheck className="w-4 h-4 mr-2" />
                                Update User
                              </>
                            ) : (
                              <>
                                <Eye className="w-4 h-4 mr-2" />
                                Preview Changes
                              </>
                            )}
                          </>
                        )}
                      </Button>
                    )}
                  </div>
                </div>
              </form>
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (mode === 'create') {
    return (
      <div className={`space-y-6 ${className}`}>
        {/* Background Decorations */}
        <div className="fixed inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl animate-pulse"></div>
          <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
          <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
        </div>
        
        <div className="relative">
          {/* Header */}
          <div className="text-center mb-8">
            <div className="relative inline-block">
              <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
                👤 Create New User
              </h1>
              <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full animate-pulse"></div>
            </div>
            <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
              Add a new user to the EPSX platform with custom roles and permissions
            </p>
          </div>
          
          {/* Progress Steps */}
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5 mb-8">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full blur-sm animate-pulse opacity-60"></div>
              <div className="flex items-center justify-between">
                <div className={`flex items-center gap-2 ${formStep >= 1 ? 'text-blue-600' : 'text-gray-400'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${formStep >= 1 ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-500'}`}>
                    {formStep > 1 ? <CheckCircle className="w-4 h-4" /> : '1'}
                  </div>
                  <span className="font-medium">Basic Info</span>
                </div>
                <div className={`h-0.5 flex-1 mx-4 ${formStep >= 2 ? 'bg-blue-500' : 'bg-gray-200'}`}></div>
                <div className={`flex items-center gap-2 ${formStep >= 2 ? 'text-blue-600' : 'text-gray-400'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${formStep >= 2 ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-500'}`}>
                    {formStep > 2 ? <CheckCircle className="w-4 h-4" /> : '2'}
                  </div>
                  <span className="font-medium">Roles & Permissions</span>
                </div>
                <div className={`h-0.5 flex-1 mx-4 ${formStep >= 3 ? 'bg-blue-500' : 'bg-gray-200'}`}></div>
                <div className={`flex items-center gap-2 ${formStep >= 3 ? 'text-blue-600' : 'text-gray-400'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center ${formStep >= 3 ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-500'}`}>
                    {previewMode ? <CheckCircle className="w-4 h-4" /> : '3'}
                  </div>
                  <span className="font-medium">Review & Create</span>
                </div>
              </div>
            </div>
          </div>

          {/* Create User Form */}
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-teal-400/20 p-0.5">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6">
              <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full blur-sm animate-pulse opacity-60"></div>
              
              <form onSubmit={createForm.handleSubmit(previewMode ? handleCreateUser : handlePreviewUser)} className="space-y-6">
                {/* Step 1: Basic Information */}
                {formStep === 1 && (
                  <div className="space-y-6">
                    <div className="flex items-center gap-2 mb-6">
                      <UserPlus className="w-6 h-6 text-green-500" />
                      <h3 className="text-xl font-semibold text-gray-800 dark:text-gray-200">Basic Information</h3>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div className="space-y-2">
                        <Label htmlFor="email" className="text-sm font-medium">
                          Email Address <span className="text-red-500">*</span>
                        </Label>
                        <div className="relative">
                          <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                          <Input
                            {...createForm.register('email')}
                            placeholder="user@example.com"
                            className="pl-12 h-12 rounded-2xl border-2 border-blue-200 dark:border-blue-700 bg-gradient-to-r from-blue-50/50 to-purple-50/50 dark:from-blue-900/20 dark:to-purple-900/20"
                          />
                        </div>
                        {createForm.formState.errors.email && (
                          <p className="text-sm text-red-500 flex items-center gap-1">
                            <AlertCircle className="w-4 h-4" />
                            {createForm.formState.errors.email.message}
                          </p>
                        )}
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="displayName" className="text-sm font-medium">
                          Display Name
                        </Label>
                        <Input
                          {...createForm.register('displayName')}
                          placeholder="John Doe"
                          className="h-12 rounded-2xl border-2 border-green-200 dark:border-green-700 bg-gradient-to-r from-green-50/50 to-emerald-50/50 dark:from-green-900/20 dark:to-emerald-900/20"
                        />
                      </div>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div className="space-y-2">
                        <Label htmlFor="firstName" className="text-sm font-medium">
                          First Name
                        </Label>
                        <Input
                          {...createForm.register('firstName')}
                          placeholder="John"
                          className="h-12 rounded-2xl border-2 border-purple-200 dark:border-purple-700"
                        />
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="lastName" className="text-sm font-medium">
                          Last Name
                        </Label>
                        <Input
                          {...createForm.register('lastName')}
                          placeholder="Doe"
                          className="h-12 rounded-2xl border-2 border-pink-200 dark:border-pink-700"
                        />
                      </div>
                    </div>
                  </div>
                )}

                {/* Step 2: Role and Package */}
                {formStep === 2 && (
                  <div className="space-y-6">
                    <div className="flex items-center gap-2 mb-6">
                      <Shield className="w-6 h-6 text-purple-500" />
                      <h3 className="text-xl font-semibold text-gray-800 dark:text-gray-200">Roles & Permissions</h3>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div className="space-y-2">
                        <Label htmlFor="role" className="text-sm font-medium">
                          User Role <span className="text-red-500">*</span>
                        </Label>
                        <Select
                          value={createForm.watch('role')}
                          onValueChange={(value: any) => createForm.setValue('role', value)}
                        >
                          <SelectTrigger className="h-12 rounded-2xl border-2 border-yellow-200 dark:border-yellow-700">
                            <SelectValue placeholder="Select user role" />
                          </SelectTrigger>
                          <SelectContent className="rounded-2xl">
                            {defaultRoles.map(role => (
                              <SelectItem key={role.value} value={role.value}>
                                {role.value === 'admin' && '👑 '}
                                {role.value === 'premium_user' && '⭐ '}
                                {role.value === 'user' && '👤 '}
                                {role.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                        {createForm.formState.errors.role && (
                          <p className="text-sm text-red-500 flex items-center gap-1">
                            <AlertCircle className="w-4 h-4" />
                            {createForm.formState.errors.role.message}
                          </p>
                        )}
                      </div>

                      <div className="space-y-2">
                        <Label htmlFor="packageTier" className="text-sm font-medium">
                          Package Tier <span className="text-red-500">*</span>
                        </Label>
                        <Select
                          value={createForm.watch('packageTier')}
                          onValueChange={(value) => createForm.setValue('packageTier', value)}
                        >
                          <SelectTrigger className="h-12 rounded-2xl border-2 border-orange-200 dark:border-orange-700">
                            <SelectValue placeholder="Select package tier" />
                          </SelectTrigger>
                          <SelectContent className="rounded-2xl">
                            {defaultTiers.map(tier => (
                              <SelectItem key={tier.value} value={tier.value}>
                                {tier.value === 'basic' && '🥉 '}
                                {tier.value === 'premium' && '🥈 '}
                                {tier.value === 'pro' && '🥇 '}
                                {tier.value === 'enterprise' && '💎 '}
                                {tier.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    </div>

                    {/* Initial Permissions */}
                    <div className="space-y-2">
                      <Label className="text-sm font-medium">Initial Permissions (Optional)</Label>
                      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-gray-100/50 to-gray-200/50 dark:from-gray-800/50 dark:to-gray-700/50 p-0.5">
                        <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-sm rounded-2xl p-4 max-h-64 overflow-y-auto">
                          {availablePermissions.length > 0 ? (
                            <div className="space-y-2">
                              {availablePermissions.map(permission => (
                                <label key={permission.id} className="flex items-start gap-3 p-3 hover:bg-gray-50 dark:hover:bg-gray-800 rounded-xl cursor-pointer transition-colors">
                                  <Checkbox
                                    checked={createForm.watch('permissions').includes(permission.id)}
                                    onCheckedChange={(checked) => {
                                      const currentPermissions = createForm.watch('permissions');
                                      if (checked) {
                                        createForm.setValue('permissions', [...currentPermissions, permission.id]);
                                      } else {
                                        createForm.setValue('permissions', currentPermissions.filter(p => p !== permission.id));
                                      }
                                    }}
                                  />
                                  <div className="flex-1">
                                    <div className="font-medium text-sm">{permission.name}</div>
                                    <div className="text-xs text-gray-500 dark:text-gray-400">{permission.description}</div>
                                    <Badge variant="secondary" size="sm" className="mt-1">
                                      {permission.category}
                                    </Badge>
                                  </div>
                                </label>
                              ))}
                            </div>
                          ) : (
                            <div className="text-center py-8">
                              <Shield className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                              <p className="text-gray-500">No permissions available to assign</p>
                            </div>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* Account Status */}
                    <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-2xl">
                      <div>
                        <Label className="text-sm font-medium">Account Status</Label>
                        <p className="text-xs text-gray-500 dark:text-gray-400">Enable this account immediately after creation</p>
                      </div>
                      <Switch
                        checked={createForm.watch('isActive')}
                        onCheckedChange={(checked) => createForm.setValue('isActive', checked)}
                      />
                    </div>
                  </div>
                )}

                {/* Step 3: Review */}
                {(formStep === 3 || previewMode) && (
                  <div className="space-y-6">
                    <div className="flex items-center gap-2 mb-6">
                      <CheckCircle className="w-6 h-6 text-green-500" />
                      <h3 className="text-xl font-semibold text-gray-800 dark:text-gray-200">
                        {previewMode ? 'Confirm User Creation' : 'Review Information'}
                      </h3>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div className="space-y-4">
                        <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-2xl">
                          <h4 className="font-semibold text-blue-800 dark:text-blue-200 mb-2">Basic Information</h4>
                          <div className="space-y-1 text-sm">
                            <p><strong>Email:</strong> {createForm.watch('email') || 'Not provided'}</p>
                            <p><strong>Display Name:</strong> {createForm.watch('displayName') || 'Not provided'}</p>
                            <p><strong>First Name:</strong> {createForm.watch('firstName') || 'Not provided'}</p>
                            <p><strong>Last Name:</strong> {createForm.watch('lastName') || 'Not provided'}</p>
                          </div>
                        </div>
                        
                        <div className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded-2xl">
                          <h4 className="font-semibold text-purple-800 dark:text-purple-200 mb-2">Access Control</h4>
                          <div className="space-y-1 text-sm">
                            <p><strong>Role:</strong> {createForm.watch('role') || 'Not selected'}</p>
                            <p><strong>Package Tier:</strong> {createForm.watch('packageTier') || 'Not selected'}</p>
                            <p><strong>Status:</strong> {createForm.watch('isActive') ? 'Active' : 'Inactive'}</p>
                          </div>
                        </div>
                      </div>
                      
                      <div className="space-y-4">
                        <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-2xl">
                          <h4 className="font-semibold text-green-800 dark:text-green-200 mb-2">Initial Permissions</h4>
                          {createForm.watch('permissions').length > 0 ? (
                            <div className="space-y-1">
                              {createForm.watch('permissions').map((permId, index) => {
                                const perm = availablePermissions.find(p => p.id === permId);
                                return (
                                  <div key={index} className="flex items-center gap-2 text-sm">
                                    <Shield className="w-3 h-3 text-green-600" />
                                    {perm?.name || permId}
                                  </div>
                                );
                              })}
                            </div>
                          ) : (
                            <p className="text-sm text-gray-500">No initial permissions selected</p>
                          )}
                        </div>
                      </div>
                    </div>
                    
                    {previewMode && (
                      <div className="p-4 bg-yellow-50 dark:bg-yellow-900/20 border-2 border-yellow-200 dark:border-yellow-700 rounded-2xl">
                        <div className="flex items-center gap-2 mb-2">
                          <AlertCircle className="w-5 h-5 text-yellow-600" />
                          <h4 className="font-semibold text-yellow-800 dark:text-yellow-200">Ready to Create</h4>
                        </div>
                        <p className="text-sm text-yellow-700 dark:text-yellow-300">
                          Please review the information above. Once you click "Create User", the account will be created and the user will be able to access the system.
                        </p>
                      </div>
                    )}
                  </div>
                )}

                {/* Navigation Buttons */}
                <div className="flex justify-between pt-6 border-t border-gray-200 dark:border-gray-700">
                  <div>
                    {formStep > 1 && !previewMode && (
                      <Button
                        type="button"
                        variant="outline"
                        onClick={handlePrevStep}
                        className="rounded-2xl"
                      >
                        Previous
                      </Button>
                    )}
                    {previewMode && (
                      <Button
                        type="button"
                        variant="outline"
                        onClick={() => setPreviewMode(false)}
                        className="rounded-2xl"
                      >
                        Back to Edit
                      </Button>
                    )}
                  </div>
                  
                  <div className="flex gap-3">
                    <Button
                      type="button"
                      variant="outline"
                      onClick={() => {
                        if (confirm('Are you sure? All entered data will be lost.')) {
                          router.push('/users');
                        }
                      }}
                      className="rounded-2xl"
                    >
                      Cancel
                    </Button>
                    
                    {formStep < 3 && !previewMode && (
                      <Button
                        type="button"
                        onClick={handleNextStep}
                        className="bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 rounded-2xl"
                      >
                        Next Step
                      </Button>
                    )}
                    
                    {(formStep === 3 || previewMode) && (
                      <Button
                        type="submit"
                        disabled={isSubmitting}
                        className={`rounded-2xl ${
                          previewMode 
                            ? 'bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600'
                            : 'bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600'
                        }`}
                      >
                        {isSubmitting ? (
                          <>
                            <Clock className="w-4 h-4 mr-2 animate-spin" />
                            {previewMode ? 'Creating...' : 'Processing...'}
                          </>
                        ) : (
                          <>
                            {previewMode ? (
                              <>
                                <CheckCircle className="w-4 h-4 mr-2" />
                                Create User
                              </>
                            ) : (
                              <>
                                <Eye className="w-4 h-4 mr-2" />
                                Preview & Confirm
                              </>
                            )}
                          </>
                        )}
                      </Button>
                    )}
                  </div>
                </div>
              </form>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Original tabbed interface for other modes
  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div>
        <h2 className="text-xl font-bold text-white mb-2">User Operations</h2>
        <p className="text-gray-400">Create, edit, and manage users in bulk</p>
      </div>

      {/* Main Form Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="create">Create User</TabsTrigger>
          <TabsTrigger value="edit">Edit User</TabsTrigger>
          <TabsTrigger value="bulk">Bulk Operations</TabsTrigger>
          <TabsTrigger value="import">Import/Export</TabsTrigger>
        </TabsList>

        {/* Create User Tab */}
        <TabsContent value="create" className="space-y-4">
          <Card className="p-6">
            <form onSubmit={createForm.handleSubmit(handleCreateUser)} className="space-y-6">
              <div className="flex items-center gap-2 mb-4">
                <UserPlus className="w-5 h-5 text-green-500" />
                <h3 className="text-lg font-medium text-white">Create New User</h3>
              </div>

              {/* Basic Information */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="email">Email <span className="text-red-500">*</span></Label>
                  <div className="relative">
                    <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                    <Input
                      {...createForm.register('email')}
                      placeholder="user@example.com"
                      className="pl-10"
                    />
                  </div>
                  {createForm.formState.errors.email && (
                    <p className="text-sm text-red-500">{createForm.formState.errors.email.message}</p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="displayName">Display Name</Label>
                  <Input
                    {...createForm.register('displayName')}
                    placeholder="John Doe"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="firstName">First Name</Label>
                  <Input
                    {...createForm.register('firstName')}
                    placeholder="John"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="lastName">Last Name</Label>
                  <Input
                    {...createForm.register('lastName')}
                    placeholder="Doe"
                  />
                </div>
              </div>

              {/* Role and Tier */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="role">Role</Label>
                  <Select
                    value={createForm.watch('role')}
                    onValueChange={(value: any) => createForm.setValue('role', value)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select role" />
                    </SelectTrigger>
                    <SelectContent>
                      {defaultRoles.map(role => (
                        <SelectItem key={role.value} value={role.value}>
                          {role.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  {createForm.formState.errors.role && (
                    <p className="text-sm text-red-500">{createForm.formState.errors.role.message}</p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="packageTier">Package Tier</Label>
                  <Select
                    value={createForm.watch('packageTier')}
                    onValueChange={(value) => createForm.setValue('packageTier', value)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select tier" />
                    </SelectTrigger>
                    <SelectContent>
                      {defaultTiers.map(tier => (
                        <SelectItem key={tier.value} value={tier.value}>
                          {tier.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>

              {/* Permissions */}
              <div className="space-y-2">
                <Label>Initial Permissions (Optional)</Label>
                <div className="border border-gray-700 rounded-md p-3 max-h-48 overflow-y-auto">
                  <div className="space-y-2">
                    {availablePermissions.map(permission => (
                      <label key={permission.id} className="flex items-start gap-3 p-2 hover:bg-gray-800 rounded cursor-pointer">
                        <Checkbox
                          checked={createForm.watch('permissions').includes(permission.id)}
                          onCheckedChange={(checked) => {
                            const currentPermissions = createForm.watch('permissions');
                            if (checked) {
                              createForm.setValue('permissions', [...currentPermissions, permission.id]);
                            } else {
                              createForm.setValue('permissions', currentPermissions.filter(p => p !== permission.id));
                            }
                          }}
                        />
                        <div className="flex-1">
                          <div className="font-medium text-sm">{permission.name}</div>
                          <div className="text-xs text-gray-400">{permission.description}</div>
                          <Badge variant="secondary" size="sm" className="mt-1">
                            {permission.category}
                          </Badge>
                        </div>
                      </label>
                    ))}
                  </div>
                </div>
              </div>

              {/* Account Status */}
              <div className="flex items-center space-x-2">
                <Switch
                  checked={createForm.watch('isActive')}
                  onCheckedChange={(checked) => createForm.setValue('isActive', checked)}
                />
                <Label>Account Active</Label>
              </div>

              {/* Submit Buttons */}
              <div className="flex justify-end gap-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => createForm.reset()}
                >
                  Reset
                </Button>
                <Button
                  type="submit"
                  disabled={isSubmitting}
                >
                  {isSubmitting ? (
                    <>
                      <Clock className="w-4 h-4 mr-2 animate-spin" />
                      Creating...
                    </>
                  ) : (
                    <>
                      <UserPlus className="w-4 h-4 mr-2" />
                      Create User
                    </>
                  )}
                </Button>
              </div>
            </form>
          </Card>
        </TabsContent>

        {/* Edit User Tab */}
        <TabsContent value="edit" className="space-y-4">
          <Card className="p-6">
            <div className="flex items-center gap-2 mb-4">
              <UserIcon className="w-5 h-5 text-blue-500" />
              <h3 className="text-lg font-medium text-white">Edit User</h3>
            </div>

            {!editForm.watch('id') ? (
              <div className="text-center py-8">
                <UserIcon className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <p className="text-gray-400">Select a user from the main table to edit</p>
              </div>
            ) : (
              <form onSubmit={editForm.handleSubmit(handleEditUser)} className="space-y-6">
                {/* Similar form fields as create, but with edit functionality */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="editEmail">Email</Label>
                    <Input
                      {...editForm.register('email')}
                      placeholder="user@example.com"
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="editDisplayName">Display Name</Label>
                    <Input
                      {...editForm.register('displayName')}
                      placeholder="John Doe"
                    />
                  </div>
                </div>

                <div className="flex justify-end gap-3">
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => editForm.reset()}
                  >
                    Cancel
                  </Button>
                  <Button
                    type="submit"
                    disabled={isSubmitting}
                  >
                    {isSubmitting ? (
                      <>
                        <Clock className="w-4 h-4 mr-2 animate-spin" />
                        Saving...
                      </>
                    ) : (
                      <>
                        <Save className="w-4 h-4 mr-2" />
                        Save Changes
                      </>
                    )}
                  </Button>
                </div>
              </form>
            )}
          </Card>
        </TabsContent>

        {/* Bulk Operations Tab */}
        <TabsContent value="bulk" className="space-y-4">
          <Card className="p-6">
            <form onSubmit={bulkForm.handleSubmit(handleBulkOperation)} className="space-y-6">
              <div className="flex items-center gap-2 mb-4">
                <Users className="w-5 h-5 text-purple-500" />
                <h3 className="text-lg font-medium text-white">Bulk User Operations</h3>
              </div>

              {/* User Selection */}
              <div className="space-y-2">
                <Label>Select Users</Label>
                <div className="border border-gray-700 rounded-md p-3 max-h-48 overflow-y-auto">
                  <div className="space-y-2">
                    {users.map(user => (
                      <label key={user.id} className="flex items-center gap-3 p-2 hover:bg-gray-800 rounded cursor-pointer">
                        <Checkbox
                          checked={selectedUsers.includes(user.id)}
                          onCheckedChange={(checked) => {
                            if (checked) {
                              setSelectedUsers([...selectedUsers, user.id]);
                              bulkForm.setValue('userIds', [...selectedUsers, user.id]);
                            } else {
                              const updated = selectedUsers.filter(id => id !== user.id);
                              setSelectedUsers(updated);
                              bulkForm.setValue('userIds', updated);
                            }
                          }}
                        />
                        <div className="flex items-center gap-2 flex-1">
                          <div className="w-6 h-6 rounded-full bg-blue-500/20 flex items-center justify-center text-xs">
                            {user.email[0].toUpperCase()}
                          </div>
                          <div>
                            <div className="text-sm font-medium">{user.email}</div>
                            <div className="text-xs text-gray-500">{user.name || 'No name'}</div>
                          </div>
                        </div>
                        <Badge variant={user.isActive ? 'default' : 'secondary'} size="sm">
                          {user.role}
                        </Badge>
                      </label>
                    ))}
                  </div>
                </div>
                {selectedUsers.length > 0 && (
                  <p className="text-sm text-blue-400">
                    {selectedUsers.length} user(s) selected
                  </p>
                )}
              </div>

              {/* Operation Selection */}
              <div className="space-y-2">
                <Label htmlFor="operation">Operation</Label>
                <Select
                  value={bulkForm.watch('operation')}
                  onValueChange={(value: any) => {
                    bulkForm.setValue('operation', value);
                    setBulkOperation(value);
                  }}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select operation" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="update_role">Update Role</SelectItem>
                    <SelectItem value="update_tier">Update Package Tier</SelectItem>
                    <SelectItem value="assign_permissions">Assign Permissions</SelectItem>
                    <SelectItem value="activate">Activate Users</SelectItem>
                    <SelectItem value="deactivate">Deactivate Users</SelectItem>
                    <SelectItem value="delete">Delete Users</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Conditional Fields Based on Operation */}
              {bulkOperation === 'update_role' && (
                <div className="space-y-2">
                  <Label htmlFor="bulkRole">New Role</Label>
                  <Select
                    value={bulkForm.watch('role') || ''}
                    onValueChange={(value) => bulkForm.setValue('role', value)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select role" />
                    </SelectTrigger>
                    <SelectContent>
                      {defaultRoles.map(role => (
                        <SelectItem key={role.value} value={role.value}>
                          {role.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}

              {bulkOperation === 'update_tier' && (
                <div className="space-y-2">
                  <Label htmlFor="bulkTier">New Package Tier</Label>
                  <Select
                    value={bulkForm.watch('packageTier') || ''}
                    onValueChange={(value) => bulkForm.setValue('packageTier', value)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select tier" />
                    </SelectTrigger>
                    <SelectContent>
                      {defaultTiers.map(tier => (
                        <SelectItem key={tier.value} value={tier.value}>
                          {tier.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}

              {/* Reason */}
              <div className="space-y-2">
                <Label htmlFor="bulkReason">Reason <span className="text-red-500">*</span></Label>
                <Textarea
                  {...bulkForm.register('reason')}
                  placeholder="Reason for bulk operation..."
                  rows={3}
                />
                {bulkForm.formState.errors.reason && (
                  <p className="text-sm text-red-500">{bulkForm.formState.errors.reason.message}</p>
                )}
              </div>

              {/* Submit Button */}
              <div className="flex justify-end gap-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    bulkForm.reset();
                    setSelectedUsers([]);
                    setBulkOperation('');
                  }}
                >
                  Reset
                </Button>
                <Button
                  type="submit"
                  disabled={isSubmitting || selectedUsers.length === 0}
                  variant={bulkOperation === 'delete' ? 'destructive' : 'default'}
                >
                  {isSubmitting ? (
                    <>
                      <Clock className="w-4 h-4 mr-2 animate-spin" />
                      Processing...
                    </>
                  ) : (
                    <>
                      <Users className="w-4 h-4 mr-2" />
                      Apply to {selectedUsers.length} User(s)
                    </>
                  )}
                </Button>
              </div>
            </form>
          </Card>
        </TabsContent>

        {/* Import/Export Tab */}
        <TabsContent value="import" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Import Section */}
            <Card className="p-6">
              <div className="flex items-center gap-2 mb-4">
                <Upload className="w-5 h-5 text-green-500" />
                <h3 className="text-lg font-medium text-white">Import Users</h3>
              </div>

              <div className="space-y-4">
                <div className="text-sm text-gray-400">
                  <p>Upload a CSV file to import multiple users at once.</p>
                  <p className="mt-1">
                    <strong>Required columns:</strong> email, role, packageTier
                  </p>
                  <p>
                    <strong>Optional columns:</strong> firstName, lastName, displayName, isActive
                  </p>
                </div>

                <div className="border-2 border-dashed border-gray-700 rounded-lg p-6 text-center">
                  <input
                    type="file"
                    accept=".csv"
                    onChange={(e) => {
                      const file = e.target.files?.[0];
                      if (file) handleCSVImport(file);
                    }}
                    className="hidden"
                    id="csv-upload"
                  />
                  <label htmlFor="csv-upload" className="cursor-pointer">
                    <FileText className="w-8 h-8 text-gray-400 mx-auto mb-2" />
                    <p className="text-sm text-gray-300">
                      Click to upload or drag and drop
                    </p>
                    <p className="text-xs text-gray-500">CSV files only</p>
                  </label>
                </div>

                {isImporting && (
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-400">Importing users...</span>
                      <span className="text-gray-300">{importProgress}%</span>
                    </div>
                    <Progress value={importProgress} className="h-2" />
                  </div>
                )}
              </div>
            </Card>

            {/* Export Section */}
            <Card className="p-6">
              <div className="flex items-center gap-2 mb-4">
                <Download className="w-5 h-5 text-blue-500" />
                <h3 className="text-lg font-medium text-white">Export Users</h3>
              </div>

              <div className="space-y-4">
                <p className="text-sm text-gray-400">
                  Export user data in various formats for backup or external processing.
                </p>

                <div className="space-y-3">
                  <Button variant="outline" className="w-full justify-start">
                    <FileText className="w-4 h-4 mr-2" />
                    Export as CSV
                  </Button>
                  <Button variant="outline" className="w-full justify-start">
                    <FileText className="w-4 h-4 mr-2" />
                    Export as JSON
                  </Button>
                  <Button variant="outline" className="w-full justify-start">
                    <FileText className="w-4 h-4 mr-2" />
                    Export as Excel
                  </Button>
                </div>

                <div className="text-xs text-gray-500 p-3 bg-gray-800/50 rounded">
                  <p><strong>Note:</strong> Exports will exclude sensitive information like passwords and include only basic user data and permissions.</p>
                </div>
              </div>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default UserForms;