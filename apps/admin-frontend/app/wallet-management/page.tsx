/**
 * Admin Wallet Management Page
 * Complete interface for managing Web3 wallet users and permissions
 */
import React from 'react';
import { WalletUserManagement } from '@/components/users/WalletUserManagement';

export default function WalletManagementPage() {
  return (
    <div className="container mx-auto py-6">
      <div className="mb-6">
        <h1 className="text-3xl font-bold">Wallet Management</h1>
        <p className="text-gray-600 mt-2">
          Manage Web3 wallet users, permissions, and group assignments
        </p>
      </div>
      
      <WalletUserManagement />
    </div>
  );
}

export const metadata = {
  title: 'Wallet Management - EPSX Admin',
  description: 'Manage Web3 wallet users and permissions',
};