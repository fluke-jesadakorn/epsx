'use client';

import React from 'react';

import type { Permission, TokenFeature } from '@/types/auth/features';
import type { UserRole } from '@/types/auth/roles';

interface User {
  userId: string;
  email?: string;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
}

interface UserManagementSectionProps {
  users: User[];
}

const UserManagementSection = ({
  users,
}: UserManagementSectionProps): React.JSX.Element => {
  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4">User Management</h2>
      <div className="overflow-x-auto">
        <table className="min-w-full bg-white">
          <thead>
            <tr>
              <th className="py-2 px-4 border-b">User ID</th>
              <th className="py-2 px-4 border-b">Email</th>
              <th className="py-2 px-4 border-b">Role</th>
              <th className="py-2 px-4 border-b">Actions</th>
            </tr>
          </thead>
          <tbody>
            {users.map((user) => (
              <tr key={user.userId}>
                <td className="py-2 px-4 border-b text-center">
                  {user.userId}
                </td>
                <td className="py-2 px-4 border-b text-center">
                  {user.email || 'N/A'}
                </td>
                <td className="py-2 px-4 border-b text-center">{user.role}</td>
                <td className="py-2 px-4 border-b text-center">
                  <button className="text-blue-500 hover:underline">
                    Edit
                  </button>
                  <button className="text-red-500 hover:underline ml-2">
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default UserManagementSection;
