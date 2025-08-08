'use client';

import { useState } from 'react';
import { Shield, Users, FileText, Settings, Search, Filter } from 'lucide-react';
import type { UserWithPermissions as ImportedUserWithPermissions, PackageTier } from '@/types/admin/iam';

interface UserWithPermissions extends ImportedUserWithPermissions {
  uid: string;
  email: string;
  name?: string;
  packageTier: PackageTier;
  status: 'active' | 'inactive' | 'disabled';
  customPermissions: any[];
}

interface Role {
  id: string;
  name: string;
  description?: string;
}

interface Policy {
  id: string;
  name: string;
  description?: string;
}

interface IAMContentProps {
  users: UserWithPermissions[];
  roles: Role[];
  policies: Policy[];
}

export function IAMContent({ users, roles, policies }: IAMContentProps) {
  const [activeTab, setActiveTab] = useState<'users' | 'roles' | 'policies' | 'audit'>('users');
  const [searchTerm, setSearchTerm] = useState('');

  const filteredUsers = users.filter(user =>
    user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
    user.name?.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const filteredRoles = roles.filter(role =>
    role.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    role.description?.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const filteredPolicies = policies.filter(policy =>
    policy.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    policy.description?.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <>
      {/* Navigation Tabs */}
      <div className="pancake-card p-1">
        <div className="flex space-x-1">
          {[
            { id: 'users', label: 'Users', icon: Users },
            { id: 'roles', label: 'Roles', icon: Shield },
            { id: 'policies', label: 'Policies', icon: FileText },
            { id: 'audit', label: 'Audit Logs', icon: Settings }
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all ${
                activeTab === tab.id
                  ? 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white'
                  : 'text-muted-foreground hover:text-foreground'
              }`}
            >
              <tab.icon className="h-4 w-4" />
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Search and Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder={`Search ${activeTab}...`}
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="form-input pl-10"
          />
        </div>
        <button className="btn-secondary flex items-center gap-2">
          <Filter className="h-4 w-4" />
          Filters
        </button>
      </div>

      {/* Content */}
      <div className="pancake-card">
        {activeTab === 'users' && (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-200 dark:border-gray-700">
                  <th className="text-left p-4">User</th>
                  <th className="text-left p-4">Package Tier</th>
                  <th className="text-left p-4">Status</th>
                  <th className="text-left p-4">Roles</th>
                  <th className="text-left p-4">Custom Permissions</th>
                  <th className="text-left p-4">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredUsers.map((user) => (
                  <tr key={user.uid} className="border-b border-gray-100 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-800/50">
                    <td className="p-4">
                      <div>
                        <div className="font-medium">{user.name || user.email}</div>
                        <div className="text-sm text-muted-foreground">{user.email}</div>
                      </div>
                    </td>
                    <td className="p-4">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                        user.packageTier === PackageTier.ENTERPRISE ? 'bg-purple-100 text-purple-800' :
                        user.packageTier === PackageTier.PLATINUM ? 'bg-yellow-100 text-yellow-800' :
                        user.packageTier === PackageTier.GOLD ? 'bg-orange-100 text-orange-800' :
                        user.packageTier === PackageTier.SILVER ? 'bg-gray-100 text-gray-800' :
                        user.packageTier === PackageTier.BRONZE ? 'bg-amber-100 text-amber-800' :
                        'bg-green-100 text-green-800'
                      }`}>
                        {user.packageTier || PackageTier.FREE}
                      </span>
                    </td>
                    <td className="p-4">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                        user.status === 'active' ? 'bg-green-100 text-green-800' :
                        user.status === 'inactive' ? 'bg-gray-100 text-gray-800' :
                        'bg-red-100 text-red-800'
                      }`}>
                        {user.status || 'active'}
                      </span>
                    </td>
                    <td className="p-4">
                      <div className="flex flex-wrap gap-1">
                        {user.roles?.slice(0, 2).map((role) => (
                          <span key={role} className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded">
                            {role}
                          </span>
                        )) || <span className="text-sm text-gray-500">No roles</span>}
                        {user.roles?.length > 2 && (
                          <span className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded">
                            +{user.roles.length - 2}
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="p-4">
                      <span className="text-sm">{user.customPermissions.length}</span>
                    </td>
                    <td className="p-4">
                      <button className="text-orange-600 hover:text-orange-800 text-sm font-medium">
                        Edit
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {activeTab === 'roles' && (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-200 dark:border-gray-700">
                  <th className="text-left p-4">Role Name</th>
                  <th className="text-left p-4">Description</th>
                  <th className="text-left p-4">Policies</th>
                  <th className="text-left p-4">Assignable</th>
                  <th className="text-left p-4">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredRoles.map((role) => (
                  <tr key={role.id} className="border-b border-gray-100 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-800/50">
                    <td className="p-4">
                      <div className="font-medium">{role.name}</div>
                    </td>
                    <td className="p-4">
                      <div className="text-sm text-muted-foreground">{role.description || 'No description'}</div>
                    </td>
                    <td className="p-4">
                      <span className="text-sm">{role.attachedPolicies?.length || 0} policies</span>
                    </td>
                    <td className="p-4">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                        role.id ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                      }`}>
                        Yes
                      </span>
                    </td>
                    <td className="p-4">
                      <button className="text-orange-600 hover:text-orange-800 text-sm font-medium">
                        Edit
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {activeTab === 'policies' && (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-200 dark:border-gray-700">
                  <th className="text-left p-4">Policy Name</th>
                  <th className="text-left p-4">Description</th>
                  <th className="text-left p-4">Type</th>
                  <th className="text-left p-4">Attached To</th>
                  <th className="text-left p-4">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredPolicies.map((policy) => (
                  <tr key={policy.id} className="border-b border-gray-100 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-800/50">
                    <td className="p-4">
                      <div className="font-medium">{policy.name}</div>
                    </td>
                    <td className="p-4">
                      <div className="text-sm text-muted-foreground">{policy.description || 'No description'}</div>
                    </td>
                    <td className="p-4">
                      <span className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded font-medium">
                        Managed
                      </span>
                    </td>
                    <td className="p-4">
                      <span className="text-sm">{policy.attachmentCount || 0} entities</span>
                    </td>
                    <td className="p-4">
                      <button className="text-orange-600 hover:text-orange-800 text-sm font-medium">
                        View
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {activeTab === 'audit' && (
          <div className="text-center py-12">
            <Settings className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
            <h3 className="text-lg font-medium mb-2">Audit Logs</h3>
            <p className="text-muted-foreground">
              Audit log functionality will be implemented here
            </p>
          </div>
        )}
      </div>
    </>
  );
}