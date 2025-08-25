import { Metadata } from 'next';
import { NewsDashboard } from '@/components/news/NewsDashboard';

export const metadata: Metadata = {
  title: 'News Management | EPSX Admin',
  description: 'Manage news articles, categories, and authors',
};

export default function NewsManagementPage() {
  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">News Management</h1>
        <p className="mt-2 text-gray-600">
          Create, edit, and manage news articles for the EPSX platform.
        </p>
      </div>
      
      <NewsDashboard />
    </div>
  );
}