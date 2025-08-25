import Link from 'next/link';
import Image from 'next/image';

interface NewsCardProps {
  article: {
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
    _sys?: {
      filename: string;
    };
  };
}

export function NewsCard({ article }: NewsCardProps) {
  // Generate article URL from filename or ID
  const urlPath = article._sys?.filename 
    ? `/news/${article._sys.filename.replace('.mdx', '')}` 
    : `/news/2025/01/article-${article.id}`;
  
  // Format date
  const publishDate = new Date(article.publishedAt).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });

  // Category colors
  const getCategoryColor = (category?: string) => {
    const colors: Record<string, string> = {
      'market-analysis': 'bg-blue-100 text-blue-800',
      'earnings-reports': 'bg-green-100 text-green-800',
      'eps-insights': 'bg-yellow-100 text-yellow-800',
      'platform-updates': 'bg-purple-100 text-purple-800',
    };
    return colors[category || ''] || 'bg-gray-100 text-gray-800';
  };

  return (
    <article className="group bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden hover:shadow-lg transition-all duration-300">
      <Link href={urlPath}>
        {/* Featured Image */}
        {article.featuredImage && (
          <div className="aspect-video overflow-hidden bg-gray-100">
            <Image
              src={article.featuredImage}
              alt={article.imageAlt || article.title}
              width={400}
              height={225}
              className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
            />
          </div>
        )}
        
        <div className="p-6">
          {/* Category Badge */}
          {article.category && (
            <div className="mb-3">
              <span className={`inline-block px-3 py-1 text-xs font-medium rounded-full ${getCategoryColor(article.category)}`}>
                {article.category.split('-').map(word => 
                  word.charAt(0).toUpperCase() + word.slice(1)
                ).join(' ')}
              </span>
            </div>
          )}
          
          {/* Title */}
          <h3 className="text-xl font-bold text-gray-900 mb-3 line-clamp-2 group-hover:text-blue-600 transition-colors">
            {article.title}
          </h3>
          
          {/* Excerpt */}
          {article.excerpt && (
            <p className="text-gray-600 text-sm leading-relaxed mb-4 line-clamp-3">
              {article.excerpt}
            </p>
          )}
          
          {/* Meta Information */}
          <div className="flex items-center justify-between text-sm text-gray-500">
            <div className="flex items-center space-x-4">
              <time dateTime={article.publishedAt}>
                {publishDate}
              </time>
              
              {article.readTime && (
                <>
                  <span>•</span>
                  <span>{article.readTime} min read</span>
                </>
              )}
            </div>
            
            {article.author && (
              <span className="text-blue-600 font-medium">
                {article.author}
              </span>
            )}
          </div>
          
          {/* Tags */}
          {article.tags && article.tags.length > 0 && (
            <div className="mt-4 pt-4 border-t border-gray-100">
              <div className="flex flex-wrap gap-2">
                {article.tags.slice(0, 3).map((tag) => (
                  <span
                    key={tag}
                    className="inline-block px-2 py-1 text-xs bg-gray-50 text-gray-600 rounded-md"
                  >
                    #{tag}
                  </span>
                ))}
                {article.tags.length > 3 && (
                  <span className="inline-block px-2 py-1 text-xs text-gray-400">
                    +{article.tags.length - 3} more
                  </span>
                )}
              </div>
            </div>
          )}
        </div>
      </Link>
    </article>
  );
}