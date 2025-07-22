'use client';

import { Edit, MoreHorizontal, Plus, Search, Shield } from 'lucide-react';
import React, { useState } from 'react';
import { useUsers } from '../../../hooks/iam/useUsers';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Badge, Button, Input } from '../../ui/form-components';
import { CreateUserModal } from './CreateUserModal';
import { UserDetailsModal } from './UserDetailsModal';

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

const InlineActions: React.FC<InlineActionsProps> = ({
  onEdit,
  onViewDetails,
}) => {
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
        <div className="absolute right-0 top-8 z-10 w-48 rounded-md border border-border bg-background shadow-lg">
          <div className="py-1">
            <button
              onClick={() => {
                onViewDetails();
                setIsOpen(false);
              }}
              className="flex w-full items-center px-4 py-2 text-sm text-foreground hover:bg-accent hover:text-accent-foreground"
            >
              <Shield className="mr-2 h-4 w-4" />
              View Details
            </button>
            <button
              onClick={() => {
                onEdit();
                setIsOpen(false);
              }}
              className="flex w-full items-center px-4 py-2 text-sm text-foreground hover:bg-accent hover:text-accent-foreground"
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

const UserCard: React.FC<UserRowProps> = ({ user, onEdit, onViewDetails }) => (
  <div className="bg-card border border-border rounded-lg p-4 space-y-3">
    <div className="flex justify-between items-start">
      <div className="flex-1">
        <div className="font-medium text-foreground">{user.name}</div>
        <div className="text-sm text-muted-foreground">{user.email}</div>
      </div>
      <InlineActions onEdit={onEdit} onViewDetails={onViewDetails} />
    </div>
    
    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
      <div>
        <div className="text-xs text-muted-foreground font-medium uppercase tracking-wide">Package</div>
        <Badge variant={user.packageTier === 'premium' ? 'default' : 'secondary'} className="mt-1">
          {user.packageTier}
        </Badge>
      </div>
      
      <div>
        <div className="text-xs text-muted-foreground font-medium uppercase tracking-wide">Status</div>
        <Badge variant={user.status === 'active' ? 'default' : 'destructive'} className="mt-1">
          {user.status}
        </Badge>
      </div>
      
      <div>
        <div className="text-xs text-muted-foreground font-medium uppercase tracking-wide">Last Active</div>
        <div className="text-sm text-foreground mt-1">{user.lastActive}</div>
      </div>
      
      <div>
        <div className="text-xs text-muted-foreground font-medium uppercase tracking-wide">Permissions</div>
        <div className="text-sm text-foreground mt-1">{user.permissions.length} permissions</div>
      </div>
    </div>
  </div>
);

export const UserManagement: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [packageFilter, setPackageFilter] = useState('all');
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [showUserModal, setShowUserModal] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);

  const { users, loading, refetch } = useUsers({
    searchTerm,
    statusFilter,
    packageFilter,
  });

  const handleViewDetails = (user: User) => {
    setSelectedUser(user);
    setShowUserModal(true);
  };

  const handleEdit = (user: User) => {
    // TODO: Implement edit functionality
    console.log('Edit user:', user);
  };

  const handleCreateUser = async (userData: any) => {
    try {
      // TODO: Implement user creation via iamService
      console.log('Creating user:', userData);
      // await iamService.createUser(userData);
      refetch();
    } catch (error) {
      console.error('Error creating user:', error);
      throw error;
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex justify-between items-center">
          <CardTitle>User Management</CardTitle>
          <Button
            onClick={() => setShowCreateModal(true)}
            className="flex items-center gap-2"
          >
            <Plus className="h-4 w-4" />
            Add User
          </Button>
        </div>
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
              className="h-10 px-3 py-2 border border-input bg-background text-foreground rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
            >
              <option value="all">All Status</option>
              <option value="active">Active</option>
              <option value="inactive">Inactive</option>
              <option value="suspended">Suspended</option>
            </select>

            <select
              value={packageFilter}
              onChange={(e) => setPackageFilter(e.target.value)}
              className="h-10 px-3 py-2 border border-input bg-background text-foreground rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
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
          <div className="space-y-4">
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-32 bg-muted animate-pulse rounded-lg" />
            ))}
          </div>
        ) : (
          <div className="space-y-4">
            {users.map((user: User) => (
              <UserCard
                key={user.id}
                user={user}
                onEdit={() => handleEdit(user)}
                onViewDetails={() => handleViewDetails(user)}
              />
            ))}
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

      <CreateUserModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onSave={handleCreateUser}
      />
    </Card>
  );
};
