import { Metadata } from 'next';
import { NewsEditor } from '@/components/news/NewsEditor';

export const metadata: Metadata = {
  title: 'Create News Article | EPSX Admin',
  description: 'Create a new news article',
};

export default function CreateNewsPage() {
  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Create News Article</h1>
        <p className="mt-2 text-gray-600">
          Create a new article with rich content and media.
        </p>
      </div>
      
      <NewsEditor mode="create" />
    </div>
  );
}