import MyDataClientWrapper from '@/components/my-data/MyDataClientWrapper';
import { RequireProfileManagement } from '@/lib/permissions/guards';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'My Data - EPSX Analytics',
  description: 'Track and analyze your portfolio with professional-grade analytics',
};

// Server-side rendering for better performance
export const dynamic = 'force-dynamic';

export default async function MyDataPage() {
  // In a real implementation, we would fetch the user's portfolio data from a database
  // For now, we're using mock data that's defined in the client component
  
  return (
    <RequireProfileManagement
      fallback={
        <div className="container mx-auto p-6 text-center">
          <div className="max-w-md mx-auto mt-20 p-8 bg-white rounded-2xl shadow-lg border border-orange-200/50">
            <div className="w-16 h-16 mx-auto mb-4 bg-gradient-to-br from-orange-400 to-yellow-400 rounded-full flex items-center justify-center">
              <span className="text-2xl">📊</span>
            </div>
            <h1 className="text-xl font-semibold text-gray-900 mb-2">Data Access Required</h1>
            <p className="text-gray-600 mb-6">You need profile management permissions to view your data.</p>
            <button 
              className="px-6 py-2 bg-gradient-to-r from-orange-500 to-yellow-500 text-white rounded-xl font-medium hover:shadow-lg transition-all"
              onClick={() => window.location.href = '/billing'}
            >
              Upgrade Plan
            </button>
          </div>
        </div>
      }
    >
      <MyDataClientWrapper />
    </RequireProfileManagement>
  );
}