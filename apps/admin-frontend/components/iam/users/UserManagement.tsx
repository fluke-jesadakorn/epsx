'use client';

import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
import { Input, Button, Badge } from '../../ui/form-components';
import { Search, Edit, Shield, MoreHorizontal } from 'lucide-react';
import { UserDetailsModal } from './UserDetailsModal';
import { useUsers } from '../../../hooks/iam/useUsers';

interface User {
  id: string;
  name: string;
  email: string;
  packageTier: string;
  status: 'active' | 'inactive' | 'suspended';
  lastActive: string;
  permissions: string[];
}

interface InlineActionsProps {
  onEdit: () => void;
  onViewDetails: () => void;
}

const InlineActions: React.FC<InlineActionsProps> = ({ onEdit, onViewDetails }) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="relative">
      <Button 
        variant="ghost" 
        size="sm" 
        onClick={() => setIsOpen(!isOpen)}
        className="h-8 w-8 p-0"
      >
        <MoreHorizontal className="h-4 w-4" />
      </Button>
      
      {isOpen && (
        <div className="absolute right-0 top-8 z-10 w-48 rounded-md border bg-white shadow-lg">
          <div className="py-1">
            <button
              onClick={() => {
                onViewDetails();
                setIsOpen(false);
              }}
              className="flex w-full items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
            >
              <Shield className="mr-2 h-4 w-4" />
              View Details
            </button>
            <button
              onClick={() => {
                onEdit();
                setIsOpen(false);
              }}
              className="flex w-full items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
            >
              <Edit className="mr-2 h-4 w-4" />
              Edit User
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

interface UserRowProps {
  user: User;
  onEdit: () => void;
  onViewDetails: () => void;
}

const UserRow: React.FC<UserRowProps> = ({ user, onEdit, onViewDetails }) => (
  <tr className="border-b">
    <td className="px-4 py-3">
      <div>
        <div className="font-medium">{user.name}</div>
        <div className="text-sm text-gray-500">{user.email}</div>
      </div>
    </td>
    <td className="px-4 py-3">
      <Badge variant={user.packageTier === 'premium' ? 'default' : 'secondary'}>
        {user.packageTier}
      </Badge>
    </td>
    <td className="px-4 py-3">
      <Badge variant={user.status === 'active' ? 'default' : 'destructive'}>
        {user.status}
      </Badge>
    </td>
    <td className="px-4 py-3 text-sm text-gray-500">
      {user.lastActive}
    </td>
    <td className="px-4 py-3 text-sm">
      {user.permissions.length} permissions
    </td>
    <td className="px-4 py-3">
      <InlineActions onEdit={onEdit} onViewDetails={onViewDetails} />
    </td>
  </tr>
);

export const UserManagement: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [packageFilter, setPackageFilter] = useState('all');
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [showUserModal, setShowUserModal] = useState(false);

  const { users, loading } = useUsers({ searchTerm, statusFilter, packageFilter });

  const handleViewDetails = (user: User) => {
    setSelectedUser(user);
    setShowUserModal(true);
  };

  const handleEdit = (user: User) => {
    // TODO: Implement edit functionality
    console.log('Edit user:', user);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>User Management</CardTitle>
        <div className="flex flex-col sm:flex-row gap-4 mt-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
            <Input
              placeholder="Search by name, email..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-10"
            />
          </div>
          
          {/* Simple select dropdowns */}
          <div className="flex gap-2">
            <select 
              value={statusFilter} 
              onChange={(e) => setStatusFilter(e.target.value)}
              className="h-10 px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="all">All Status</option>
              <option value="active">Active</option>
              <option value="inactive">Inactive</option>
              <option value="suspended">Suspended</option>
            </select>
            
            <select 
              value={packageFilter} 
              onChange={(e) => setPackageFilter(e.target.value)}
              className="h-10 px-3 py-2 border border-gray-300 rounded-md text-sm"
            >
              <option value="all">All Packages</option>
              <option value="free">Free</option>
              <option value="premium">Premium</option>
              <option value="enterprise">Enterprise</option>
            </select>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="space-y-3">
            {[1, 2, 3].map(i => (
              <div key={i} className="h-16 bg-gray-100 animate-pulse rounded" />
            ))}
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b">
                  <th className="text-left px-4 py-3 font-medium">User</th>
                  <th className="text-left px-4 py-3 font-medium">Package</th>
                  <th className="text-left px-4 py-3 font-medium">Status</th>
                  <th className="text-left px-4 py-3 font-medium">Last Active</th>
                  <th className="text-left px-4 py-3 font-medium">Permissions</th>
                  <th className="text-left px-4 py-3 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {users.map((user: User) => (
                  <UserRow
                    key={user.id}
                    user={user}
                    onEdit={() => handleEdit(user)}
                    onViewDetails={() => handleViewDetails(user)}
                  />
                ))}
              </tbody>
            </table>
          </div>
        )}
      </CardContent>

      {showUserModal && selectedUser && (
        <UserDetailsModal
          user={selectedUser}
          open={showUserModal}
          onClose={() => setShowUserModal(false)}
        />
      )}
    </Card>
  );
};
