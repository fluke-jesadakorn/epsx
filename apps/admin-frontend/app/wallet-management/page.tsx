/**
 * Admin Wallet Management Page
 * Complete interface for managing Web3 wallet users and permissions
 */
'use client';

import { Search, User } from 'lucide-react';
import React, { useState } from 'react';

import { EnhancedWalletSearch } from '@/components/admin/EnhancedWalletSearch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { WalletUserManagement } from '@/components/users/WalletUserManagement';

/**
 *
 */
export default function WalletManagementPage() {
  const [activeTab, setActiveTab] = useState('search');

  return (
    <div className="container mx-auto py-6">
      <div className="mb-6">
        <h1 className="text-3xl font-bold">Wallet Management</h1>
        <p className="text-gray-600 mt-2">
          Manage Web3 wallet users, permissions, and group assignments
        </p>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="search" className="flex items-center gap-2">
            <Search className="h-4 w-4" />
            Advanced Search
          </TabsTrigger>
          <TabsTrigger value="lookup" className="flex items-center gap-2">
            <User className="h-4 w-4" />
            Individual Lookup
          </TabsTrigger>
        </TabsList>
        
        <TabsContent value="search" className="mt-6">
          <EnhancedWalletSearch />
        </TabsContent>
        
        <TabsContent value="lookup" className="mt-6">
          <WalletUserManagement />
        </TabsContent>
      </Tabs>
    </div>
  );
}