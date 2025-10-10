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
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-cyan-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-green-400/15 to-emerald-500/15 rounded-full blur-xl"></div>
      </div>

      <div className="relative container mx-auto px-4 py-8 max-w-7xl">
        <div className="mb-8 text-center sm:text-left">
          <div className="relative inline-block">
            <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-blue-600 via-cyan-600 to-teal-600 bg-clip-text text-transparent mb-4">
              👛 Wallet Management
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-blue-400 to-cyan-500 rounded-full"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto sm:mx-0">
            Manage Web3 wallet users, permissions, and group assignments
          </p>
        </div>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full max-w-md grid-cols-2 mb-8 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-2 border-blue-300/50 dark:border-blue-700/50 p-1">
            <TabsTrigger value="search" className="flex items-center gap-2 data-[state=active]:bg-gradient-to-r data-[state=active]:from-blue-400 data-[state=active]:to-cyan-500 data-[state=active]:text-white">
              <Search className="h-4 w-4" />
              Advanced Search
            </TabsTrigger>
            <TabsTrigger value="lookup" className="flex items-center gap-2 data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-400 data-[state=active]:to-pink-500 data-[state=active]:text-white">
              <User className="h-4 w-4" />
              Individual Lookup
            </TabsTrigger>
          </TabsList>

          <TabsContent value="search">
            <EnhancedWalletSearch />
          </TabsContent>

          <TabsContent value="lookup">
            <WalletUserManagement />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}