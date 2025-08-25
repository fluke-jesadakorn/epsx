import { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { NewsEditor } from '@/components/news/NewsEditor';

interface EditNewsPageProps {
  params: {
    id: string;
  };
}

export async function generateMetadata({ params }: EditNewsPageProps): Promise<Metadata> {
  return {
    title: `Edit Article | EPSX Admin`,
    description: 'Edit news article content and settings',
  };
}

export default function EditNewsPage({ params }: EditNewsPageProps) {
  // Decode the article ID (it will be URL encoded)
  const articleId = decodeURIComponent(params.id);
  
  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Edit News Article</h1>
        <p className="mt-2 text-gray-600">
          Edit article content and settings.
        </p>
      </div>
      
      <NewsEditor 
        mode="edit" 
        articleId={articleId}
      />
    </div>
  );
}