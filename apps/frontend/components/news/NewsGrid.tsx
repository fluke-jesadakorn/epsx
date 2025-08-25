import Link from 'next/link';
import Image from 'next/image';
import { NewsCard } from './NewsCard';

interface NewsGridProps {
  articles: Array<{
    node: {
      id: string;
      title: string;
      excerpt?: string;
      publishedAt: string;
      author?: string;
      category?: string;
      tags?: string[];
      featuredImage?: string;
      imageAlt?: string;
      readTime?: number;
      _sys: {
        filename: string;
      };
    };
  }>;
}

export function NewsGrid({ articles }: NewsGridProps) {
  if (articles.length === 0) {
    return (
      <div className="text-center py-16">
        <div className="max-w-md mx-auto">
          <div className="mb-6">
            <div className="w-24 h-24 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-12 h-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
              </svg>
            </div>
          </div>
          <h3 className="text-lg font-medium text-gray-900 mb-2">No articles found</h3>
          <p className="text-gray-600 mb-6">
            There are no published articles to display at this time.
          </p>
          <Link 
            href="/news"
            className="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
          >
            View All News
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
      {articles.map(({ node }) => (
        <NewsCard key={node.id} article={node} />
      ))}
    </div>
  );
}