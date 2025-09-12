'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { APIKeyManager } from './APIKeyManager';
import { APIDocumentation } from './APIDocumentation';
import { UsageMonitor } from './UsageMonitor';
import type { User } from '@/lib/server-actions';

interface DeveloperAPIClientProps {
  currentUser: User;
}

export function DeveloperAPIClient({ currentUser }: DeveloperAPIClientProps) {
  const [activeTab, setActiveTab] = useState('keys');

  return (
    <div className="space-y-8">
      {/* Quick Stats */}
      <div className="grid md:grid-cols-3 gap-6">
        <Card className="bg-white dark:bg-gray-800 shadow-lg">
          <CardContent className="p-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-emerald-600 mb-2">API Access</div>
              <div className="text-gray-600 dark:text-gray-300">
                {currentUser.role === 'premium' || currentUser.role === 'admin' ? 'Active' : 'Upgrade Required'}
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="bg-white dark:bg-gray-800 shadow-lg">
          <CardContent className="p-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600 mb-2">Rate Limit</div>
              <div className="text-gray-600 dark:text-gray-300">
                {currentUser.role === 'admin' ? 'Unlimited' : 
                 currentUser.role === 'premium' ? '1000/hour' : '100/hour'}
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="bg-white dark:bg-gray-800 shadow-lg">
          <CardContent className="p-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-purple-600 mb-2">Features</div>
              <div className="text-gray-600 dark:text-gray-300">
                {currentUser.role === 'admin' ? 'All Access' :
                 currentUser.role === 'premium' ? 'Advanced' : 'Basic'}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-lg">
          <div className="border-b border-gray-200 dark:border-gray-700">
            <TabsList className="w-full justify-start bg-transparent p-0 h-auto">
              <TabsTrigger 
                value="keys" 
                className="py-4 px-6 border-b-2 border-transparent data-[state=active]:border-emerald-500 data-[state=active]:text-emerald-600 dark:data-[state=active]:text-emerald-400 rounded-none bg-transparent hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                <span className="mr-2">🔐</span>
                API Keys
              </TabsTrigger>
              <TabsTrigger 
                value="docs"
                className="py-4 px-6 border-b-2 border-transparent data-[state=active]:border-emerald-500 data-[state=active]:text-emerald-600 dark:data-[state=active]:text-emerald-400 rounded-none bg-transparent hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                <span className="mr-2">📚</span>
                Documentation
              </TabsTrigger>
              <TabsTrigger 
                value="usage"
                className="py-4 px-6 border-b-2 border-transparent data-[state=active]:border-emerald-500 data-[state=active]:text-emerald-600 dark:data-[state=active]:text-emerald-400 rounded-none bg-transparent hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                <span className="mr-2">📊</span>
                Usage & Monitoring
              </TabsTrigger>
            </TabsList>
          </div>

          <div className="p-6">
            <TabsContent value="keys" className="mt-0">
              <APIKeyManager currentUser={currentUser} />
            </TabsContent>
            <TabsContent value="docs" className="mt-0">
              <APIDocumentation currentUser={currentUser} />
            </TabsContent>
            <TabsContent value="usage" className="mt-0">
              <UsageMonitor currentUser={currentUser} />
            </TabsContent>
          </div>
        </div>
      </Tabs>
    </div>
  );
}