import { NewsCard } from './NewsCard';

interface RelatedArticlesProps {
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
  currentArticleId: string;
}

export function RelatedArticles({ articles, currentArticleId }: RelatedArticlesProps) {
  // Filter out current article
  const relatedArticles = articles.filter(({ node }) => node.id !== currentArticleId);

  if (relatedArticles.length === 0) {
    return null;
  }

  return (
    <section className="bg-gray-50 py-16">
      <div className="container mx-auto px-4">
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold text-gray-900 mb-4">
            Related Articles
          </h2>
          <p className="text-gray-600 max-w-2xl mx-auto">
            Continue reading with these related articles from our market experts.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 max-w-6xl mx-auto">
          {relatedArticles.slice(0, 3).map(({ node }) => (
            <NewsCard key={node.id} article={node} />
          ))}
        </div>

        {relatedArticles.length > 3 && (
          <div className="text-center mt-12">
            <a
              href="/news"
              className="inline-flex items-center px-6 py-3 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 transition-colors"
            >
              View More Articles
              <svg className="ml-2 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </a>
          </div>
        )}
      </div>
    </section>
  );
}