import { getCurrentUser } from '@/lib/server-actions';
import { DeveloperAPIClient } from '@/components/developer/DeveloperAPIClient';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';

export const dynamic = 'force-dynamic';

export default async function DeveloperPage() {
  const user = await getCurrentUser();
  
  if (!user) {
    return (
      <div className="container mx-auto p-6">
        <h1 className="text-3xl font-bold mb-6">Developer API</h1>
        <Card>
          <CardHeader>
            <CardTitle>Authentication Required</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="mb-4">Please connect your wallet to access the Developer API portal and manage your API keys.</p>
            <p className="text-sm text-gray-500">Use the wallet button in the navigation menu to connect.</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900">
      <div className="container mx-auto px-4 py-12">
        {/* Hero Section */}
        <div className="text-center mb-16">
          <h1 className="text-4xl md:text-6xl font-bold bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-transparent mb-6">
            Developer API Portal
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto leading-relaxed">
            Access EPSX analytics data programmatically with our REST API. Manage your API keys, monitor usage, and integrate our powerful financial data into your applications.
          </p>
        </div>

        {/* Developer API Client Component */}
        <DeveloperAPIClient currentUser={user} />
      </div>
    </div>
  );
}